//! The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

#![deny(missing_docs, unstable_features)]
// #![doc = include_str!("../README.md")]

mod hash_model;
mod redis_model;
mod redissearch_model;
mod shared;

pub use redis;
pub use redis::{Client, FromRedisValue, RedisError, RedisResult, ToRedisArgs};

pub use redis_om_macros::HashModel;
pub use redis_om_macros::RedisModel;
pub use redis_om_macros::RedisSearchModel;
pub use redis_om_macros::RedisTransportValue;

pub use hash_model::HashModel;
pub use redis_model::RedisModel;
pub use redissearch_model::RedisSearchModel;
