//! The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

#![deny(missing_docs, unstable_features)]
// #![doc = include_str!("../README.md")]

mod hash_model;
mod redis_model;
mod shared;

pub use redis;

pub use redis_om_macros::HashModel;
pub use redis_om_macros::RedisModel;
pub use redis_om_macros::RedisTransportValue;

pub use hash_model::HashModel;
pub use redis_model::RedisModel;

/// A trait for types that can be transported over Redis, providing a common interface for serializing and deserializing values over Redis.
///
/// This implemented by default for types that implements `redis::ToRedisArgs` and `redis::FromRedisValue`.
///
pub trait RedisTransportValue: redis::ToRedisArgs + redis::FromRedisValue {
    /// Convert Self to redis args
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        redis::ToRedisArgs::to_redis_args(self)
    }

    /// Parse Self from redis value
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        <Self as redis::FromRedisValue>::from_redis_value(v)
    }
}

impl<T> RedisTransportValue for T where T: redis::ToRedisArgs + redis::FromRedisValue {}
