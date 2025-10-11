use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex}};

use guardian::ArcMutexGuardian;
use include_dir::{include_dir, Dir};
use postgres::Migrator;
use serde::Serialize;
use sqlformat::{Dialect, FormatOptions, QueryParams};
use sqlx::{
    error::BoxDynError,
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use sqlx_migrator::migrator::{Migrate, Plan};
use test_context::AsyncTestContext;
use tracing::{field::{Field, Visit}, span::{Attributes, Id}, subscriber::Interest, Event, Level, Metadata, Span, Subscriber};
use tracing_subscriber::{layer::{Context, SubscriberExt}, registry::LookupSpan, util::SubscriberInitExt};
use uuid::Uuid;

mod external_services;
mod media;
mod replicas;
mod sources;
mod tag_types;
mod tags;

const FIXTURES: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures");

static LAYER: LazyLock<Layer> = LazyLock::new(|| {
    let layer = Layer::new("sqlx::query");
    tracing_subscriber::registry().with(layer.clone()).try_init().unwrap();
    layer
});

pub(crate) struct DatabaseContext {
    pub layer: Layer,
    pub span: Span,
    pub conn: PgConnection,
    pub pool: PgPool,
    pub name: String,
}

async fn create_database(name: &str) -> Result<PgConnection, BoxDynError> {
    let mut conn = PgConnection::connect_with(&PgConnectOptions::new()).await?;
    conn.execute(&*format!(r#"CREATE DATABASE "{name}""#)).await?;

    Ok(conn)
}

async fn connect_database(name: &str) -> Result<PgPool, BoxDynError> {
    let connect_options = PgConnectOptions::new().database(name);
    let pool = PgPoolOptions::new()
        .connect_with(connect_options)
        .await?;

    Ok(pool)
}

async fn setup_database(pool: &PgPool) -> Result<(), BoxDynError> {
    let mut conn = pool.acquire().await?;

    let migrator = Migrator::new()?.into_boxed_migrator();
    migrator.run(&mut *conn, &Plan::apply_all()).await?;

    for file in FIXTURES.files() {
        let sql = file.contents_utf8().ok_or("invalid fixture")?;
        conn
            .execute(sql)
            .await
            .map_err(|e| format!("initializing test database failed in {:?}: {e}", file.path()))?;
    }

    Ok(())
}

async fn teardown_database(pool: PgPool) -> Result<(), BoxDynError> {
    let mut conn = pool.acquire().await?;

    let migrator = Migrator::new()?.into_boxed_migrator();
    migrator.run(&mut *conn, &Plan::revert_all()).await?;

    Ok(())
}

async fn drop_database(conn: &mut PgConnection, name: &str) -> Result<(), BoxDynError> {
    conn.execute(&*format!(r#"DROP DATABASE "{name}" WITH (FORCE)"#)).await?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Query {
    sql: String,
    rows_affected: u64,
    rows_returned: u64,
}

impl Query {
    fn new(sql: String, rows_affected: u64, rows_returned: u64) -> Self {
        Self {
            sql,
            rows_affected,
            rows_returned,
        }
    }
}

#[derive(Debug, Serialize)]
struct Queries {
    queries: Vec<Query>,
}

impl Queries {
    fn new(queries: Vec<Query>) -> Self {
        Self {
            queries,
        }
    }
}

type EventEntry = HashMap<&'static str, String>;

struct Events<'a> {
    id: &'a Id,
    inner: ArcMutexGuardian<HashMap<Id, Vec<EventEntry>>>,
}

impl<'a> Events<'a> {
    fn new(id: &'a Id, events: Arc<Mutex<HashMap<Id, Vec<EventEntry>>>>) -> Self {
        Self {
            id,
            inner: ArcMutexGuardian::take(events).unwrap(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = &'_ EventEntry> {
        self.inner
            .get(self.id)
            .into_iter()
            .flat_map(|i| i.iter())
    }
}

#[derive(Clone, Debug, Default)]
struct Layer {
    target: &'static str,
    spans: Arc<Mutex<HashMap<Id, Id>>>,
    events: Arc<Mutex<HashMap<Id, Vec<EventEntry>>>>,
}

impl Layer {
    fn new(target: &'static str) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    fn events<'a>(&self, id: &'a Id) -> Events<'a> {
        Events::new(id, self.events.clone())
    }
}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        Interest::always()
    }

    fn on_new_span(&self, _attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(parent) = ctx.current_span().id().cloned() else { return };

        let mut spans = self.spans.lock().unwrap();
        spans.insert(id.clone(), parent);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        if event.metadata().target() != self.target {
            return;
        }

        let Some(mut id) = ctx.current_span().id().cloned() else { return };

        let mut visitor = Visitor::default();
        event.record(&mut visitor);

        let spans = self.spans.lock().unwrap();
        while let Some(parent) = spans.get(&id) {
            id = parent.clone();
        }

        let mut events = self.events.lock().unwrap();
        events.entry(id).or_default().push(visitor.0);
    }
}

#[derive(Default)]
struct Visitor(HashMap<&'static str, String>);

impl Visit for Visitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name(), value.to_string());
    }
}

impl DatabaseContext {
    pub fn format(&self, sql: &str) -> String {
        sqlformat::format(sql, &QueryParams::None, &FormatOptions {
            joins_as_top_level: true,
            dialect: Dialect::PostgreSql,
            ..Default::default()
        })
    }

    pub fn queries(&self) -> Queries {
        let id = self.span.id().unwrap();
        let queries = self.layer
            .events(&id)
            .iter()
            .map(|l| {
                let statement = l.get("db.statement").map(String::as_ref);
                let summary = l.get("summary").map(String::as_ref);
                let rows_affected = l.get("rows_affected").unwrap().parse().unwrap();
                let rows_returned = l.get("rows_returned").unwrap().parse().unwrap();
                let sql = match (statement, summary) {
                    (Some("") | None, Some(summary)) => summary,
                    (Some(statement), _) => statement,
                    (None, None) => "",
                };
                Query::new(self.format(sql), rows_affected, rows_returned)
            })
            .collect();

        Queries::new(queries)
    }
}

impl AsyncTestContext for DatabaseContext {
    async fn setup() -> Self {
        let layer = LAYER.clone();
        let span = tracing::span!(Level::DEBUG, "root");

        let name = format!("hoarder_{}", Uuid::new_v4());
        let mut conn = create_database(&name).await.unwrap();

        let pool = match connect_database(&name).await {
            Ok(pool) => pool,
            Err(e) => {
                drop_database(&mut conn, &name).await.unwrap();
                panic!("{e:?}");
            },
        };

        match setup_database(&pool).await {
            Ok(()) => Self { layer, span, conn, pool, name },
            Err(e) => {
                pool.close().await;
                drop_database(&mut conn, &name).await.unwrap();
                panic!("{e:?}");
            },
        }
    }

    async fn teardown(mut self) {
        match teardown_database(self.pool).await {
            Ok(()) => {
                drop_database(&mut self.conn, &self.name).await.unwrap();
            },
            Err(e) => {
                drop_database(&mut self.conn, &self.name).await.unwrap();
                panic!("{e:?}");
            },
        }
    }
}
