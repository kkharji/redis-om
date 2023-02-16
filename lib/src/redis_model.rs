use crate::shared::{Commands, Conn};
use crate::RedisTransportValue;
use redis::RedisResult;
use std::time::Duration;

/// Get key "{self::redis_key}:{pk}"
fn fmt_key<M: RedisModel>(pk: &str) -> String {
    format!("{}:{}", M::redis_key(), pk)
}

/// Get key "{self::redis_key}:{self._get_primary_key()}"
fn fmt_model_key<M: RedisModel>(model: &M) -> String {
    fmt_key::<M>(model._get_primary_key())
}

/// Shared Redis Object Model
pub trait RedisModel: RedisTransportValue {
    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn redis_key() -> &'static str;

    /// Get primary key
    fn _get_primary_key(&self) -> &str;

    /// Get primary key
    fn _set_primary_key(&mut self, pk: String);

    /// Ensure primary key
    fn _ensure_primary_key(&mut self) {
        if self._get_primary_key() == "" {
            self._set_primary_key(rusty_ulid::generate_ulid_string())
        }
    }

    /// Expire Self at given duration
    fn expire(&self, duration: Duration, conn: &mut Conn) -> RedisResult<()> {
        let key = fmt_model_key(self);
        let seconds = duration.as_secs() as usize;

        conn.expire(key, seconds)
    }
}
