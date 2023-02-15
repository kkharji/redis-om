//! The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

#![deny(missing_docs, unstable_features)]
// #![doc = include_str!("../README.md")]

pub use redis_om_macros::RedisTransportValue;

pub use redis;

/// A trait for types that can be transported over Redis.
///
/// This trait is a combination of `redis::ToRedisArgs` and `redis::FromRedisValue`, providing a common interface for serializing and deserializing values over Redis.
///
/// The trait delegate it's default implementation  to `redis::ToRedisArgs` and `redis::FromRedisValue` traits.
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
