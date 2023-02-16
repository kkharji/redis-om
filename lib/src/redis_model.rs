use crate::shared::{Commands, Conn};
use redis::RedisResult;
use std::time::Duration;

/// Shared Redis Object Model
pub trait RedisModel: redis::ToRedisArgs + redis::FromRedisValue {
    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn redis_prefix() -> &'static str;

    /// Get primary key
    fn _get_pk(&self) -> &str;

    /// Get primary key
    fn _set_pk(&mut self, pk: String);

    /// Ensure primary key
    fn _ensure_pk(&mut self) {
        if self._get_pk() == "" {
            self._set_pk(rusty_ulid::generate_ulid_string())
        }
    }

    /// Get key "{self::redis_key}:{pk}"
    fn _fmt_pk(pk: &str) -> String {
        format!("{}:{}", Self::redis_prefix(), pk)
    }

    /// Get key "{self::redis_key}:{self._get_primary_key()}"
    fn _get_redis_key(&self) -> String {
        Self::_fmt_pk(self._get_pk())
    }

    /// Expire Self at given duration
    fn expire(&self, duration: Duration, conn: &mut Conn) -> RedisResult<()> {
        let key = self._get_redis_key();
        let seconds = duration.as_secs() as usize;

        conn.expire(key, seconds)
    }
}
