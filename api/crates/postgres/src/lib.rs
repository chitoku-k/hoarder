#![allow(clippy::enum_variant_names)]

use sea_query::Order;

use domain::repository;

mod expr;

pub mod external_services;
pub mod media;
pub mod replicas;
pub mod sources;
pub mod tag_types;
pub mod tags;

macro_rules! sea_query_uuid_value {
    ($newtype:ty, $innertype:ty) => {
        use ::sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr};
        use ::sqlx::{
            decode::Decode,
            encode::{Encode, IsNull},
            error::BoxDynError,
            postgres::{self, PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef},
            Database, Type,
        };
        use ::uuid::Uuid;

        impl From<$newtype> for Uuid {
            fn from(x: $newtype) -> Self {
                *x.0
            }
        }

        impl From<$newtype> for Value {
            fn from(x: $newtype) -> Self {
                From::<Uuid>::from(x.into())
            }
        }

        impl ValueType for $newtype {
            fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
                <Uuid as ValueType>::try_from(v).map(<$innertype>::from).map(<$newtype>::from)
            }

            fn type_name() -> String {
                <Uuid as ValueType>::type_name()
            }

            fn array_type() -> ArrayType {
                <Uuid as ValueType>::array_type()
            }

            fn column_type() -> ColumnType {
                <Uuid as ValueType>::column_type()
            }
        }

        impl Nullable for $newtype {
            fn null() -> Value {
                <Uuid as Nullable>::null()
            }
        }

        impl Type<postgres::Postgres> for $newtype {
            fn type_info() -> <postgres::Postgres as Database>::TypeInfo {
                <Uuid as Type<postgres::Postgres>>::type_info()
            }
        }

        impl PgHasArrayType for $newtype {
            fn array_type_info() -> PgTypeInfo {
                <Uuid as PgHasArrayType>::array_type_info()
            }
        }

        impl Encode<'_, postgres::Postgres> for $newtype {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                <Uuid as Encode<'_, postgres::Postgres>>::encode_by_ref(&*self.0, buf)
            }
        }

        impl Decode<'_, postgres::Postgres> for $newtype {
            fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
                <Uuid as Decode<'_, postgres::Postgres>>::decode(value).map(<$innertype>::from).map(<$newtype>::from)
            }
        }
    };
}

pub(crate) use sea_query_uuid_value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrderDirection {
    Ascending,
    Descending,
}

impl From<repository::OrderDirection> for OrderDirection {
    fn from(direction: repository::OrderDirection) -> Self {
        match direction {
            repository::OrderDirection::Ascending => OrderDirection::Ascending,
            repository::OrderDirection::Descending => OrderDirection::Descending,
        }
    }
}

impl From<OrderDirection> for Order {
    fn from(direction: OrderDirection) -> Self {
        match direction {
            OrderDirection::Ascending => Order::Asc,
            OrderDirection::Descending => Order::Desc,
        }
    }
}
