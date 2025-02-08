use clap::ValueEnum;
use tracing::Subscriber;
use tracing_subscriber::{fmt::{self, time::FormatTime, FormatFields, MakeWriter}, registry::LookupSpan, Layer};

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Compact,
    Pretty,
}

pub(super) trait LayerFormatExt<S>
where
    S: Subscriber,
{
    fn with_format(self, format: &Format) -> Box<dyn Layer<S> + Send + Sync + 'static>
    where
        Self: Sized + Layer<S> + Send + Sync + 'static;
}

impl<S, N, L, T, W> LayerFormatExt<S> for fmt::Layer<S, N, fmt::format::Format<L, T>, W>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + Send + Sync,
    T: FormatTime + Send + Sync,
    W: for<'writer> MakeWriter<'writer> + Send + Sync,
{
    fn with_format(self, format: &Format) -> Box<dyn Layer<S> + Send + Sync + 'static>
    where
        Self: Sized + Layer<S> + Send + Sync + 'static,
    {
        match format {
            Format::Compact => self.compact().boxed(),
            Format::Pretty => self.pretty().boxed(),
        }
    }
}
