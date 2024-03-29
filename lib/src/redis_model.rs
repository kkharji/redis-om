use redis::RedisResult;

/// Shared Redis Object Model
pub trait RedisModel {
    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn _prefix_key() -> &'static str;

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
        format!("{}:{}", Self::_prefix_key(), pk)
    }

    /// Check if str is  of format "{self::redis_key}:{pk}"
    fn _is_pk_fmt(pk: &str) -> bool {
        pk.starts_with(Self::_prefix_key())
    }

    /// Get key "{self::redis_key}:{self._get_primary_key()}"
    fn _get_redis_key(&self) -> String {
        Self::_fmt_pk(self._get_pk())
    }

    /// Expire Self at given duration
    fn _expire_cmd(&self, secs: usize) -> RedisResult<redis::Cmd> {
        let key = self._get_redis_key();
        let mut cmd = redis::cmd("EXPIRE");

        cmd.arg(key).arg(secs);

        Ok(cmd)
    }
}
