//! The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

#![deny(missing_docs, unstable_features)]
#![doc = include_str!("../README.md")]

mod hash_model;
#[cfg(feature = "json")]
mod json_model;
mod redis_model;
mod redissearch_model;
mod shared;
mod stream_model;

pub use redis;
pub use redis::{Client, FromRedisValue, RedisError, RedisResult, ToRedisArgs};

pub use redis_om_macros::HashModel;
#[cfg(feature = "json")]
pub use redis_om_macros::JsonModel;
pub use redis_om_macros::RedisModel;
pub use redis_om_macros::StreamModel;

/// Derive procedural macro that automatically generate implementation for
///
/// [`RedisTransportValue`](), which requires implementation of the following:
///
/// - [`ToRedisArgs`](redis::ToRedisArgs),
/// - [`FromRedisValue`](redis::FromRedisValue)
///
/// # Example
///
/// ```
/// #[derive(redis_om::RedisTransportValue)]
/// struct Test {
///     #[redis(primary_key)] // required if no `id` field exists
///     field_one: String,
///     field_two: i32,
///     field_three: Vec<u8>,
/// }
/// ```
///
/// # Attributes
///
/// ## `rename_all = "some_case"`
///
/// This attribute sets the default casing to be used for field names when generating Redis command arguments.
/// Possible values are:
///
/// - `"snake_case"`: field names will be converted to `snake_case`
/// - `"kebab-case"`: field names will be converted to `kebab-case`
/// - `"camelCase"`: field names will be converted to `camelCase`
/// - `"PascalCase"`: field names will be converted to `PascalCase`
///
/// This attribute can be overridden for individual fields using the `rename` attribute.
///
/// ## `rename = "some_field"`
///
/// This attribute sets the Redis key name to be used for a field when generating Redis command arguments.
/// This attribute takes precedence over the `rename_all` attribute.
///
/// # Restrictions
///
/// - Enums with fields are not supported
/// - Generics are not supported
/// - Public fields are required
/// - Fields with types that do not implement `ToRedisArgs` and `FromRedisValue` are not supported
pub use redis_om_macros::RedisTransportValue;

pub use hash_model::HashModel;
#[cfg(feature = "json")]
pub use json_model::JsonModel;
pub use redis_model::RedisModel;
pub use redissearch_model::RedisSearchModel;
pub use stream_model::StreamModel;
