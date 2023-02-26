use crate::redis_model::RedisModel;
use crate::shared::{Commands, Conn};
use crate::RedisSearchModel;
use ::redis::RedisResult;

/// Hash Object Model
pub trait HashModel: RedisModel + RedisSearchModel {
    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn redis_prefix() -> &'static str {
        <Self as RedisModel>::_prefix_key()
    }

    /// Save Self into redis database
    fn save(&mut self, conn: &mut Conn) -> RedisResult<()> {
        self._ensure_pk();

        let key = self._get_redis_key();
        let data = redis::ToRedisArgs::to_redis_args(self);

        redis::cmd("HSET").arg(key).arg(data).query(conn)
    }

    /// Get a list of all primary keys for current type
    fn all_pks(conn: &mut Conn) -> RedisResult<redis::Iter<'_, String>> {
        let pattern = format!("{}:*", Self::redis_prefix());

        conn.scan_match(pattern)
    }

    /// Get a list of all primary keys for current type
    fn get(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<Self> {
        let pk = pk.as_ref();
        if Self::_is_pk_fmt(pk) {
            conn.hgetall(pk)
        } else {
            conn.hgetall(Self::_fmt_pk(pk))
        }
    }

    /// Delete by given pk
    fn delete(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<()> {
        let pk = pk.as_ref();
        if Self::_is_pk_fmt(pk) {
            conn.del(pk)
        } else {
            conn.del(Self::_fmt_pk(pk))
        }
    }

    /// Redis search schema
    fn redissearch_schema() -> &'static str {
        <Self as RedisSearchModel>::REDIS_SEARCH_SCHEMA
    }

    /// Expire Self at given duration
    fn expire(&self, secs: usize, conn: &mut Conn) -> RedisResult<()> {
        self._expire(secs, conn)
    }
}
