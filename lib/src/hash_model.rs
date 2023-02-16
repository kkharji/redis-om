use crate::redis_model::RedisModel;
use crate::shared::{Commands, Conn};
use ::redis::RedisResult;

/// Get key "{self::redis_key}:{pk}"
fn fmt_key<M: HashModel>(pk: &str) -> String {
    format!("{}:{}", M::redis_key(), pk)
}

/// Get key "{self::redis_key}:{self._get_primary_key()}"
fn fmt_model_key<M: HashModel>(model: &M) -> String {
    fmt_key::<M>(model._get_primary_key())
}

/// Hash Object Model
pub trait HashModel: RedisModel {
    /// Save Self into redis database
    fn save(&mut self, conn: &mut Conn) -> RedisResult<()> {
        self._ensure_primary_key();

        let key = fmt_model_key(self);
        let data = redis::ToRedisArgs::to_redis_args(self);

        redis::cmd("HSET").arg(key).arg(data).query(conn)
    }

    /// Get a list of all primary keys for current type
    fn all_pks(conn: &mut Conn) -> RedisResult<redis::Iter<'_, String>> {
        let pattern = format!("{}:*", Self::redis_key());

        conn.scan_match(pattern)
    }

    /// Get a list of all primary keys for current type
    fn get(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<Self> {
        let key = fmt_key::<Self>(pk.as_ref());

        conn.hgetall::<_, Self>(key)
    }

    /// Delete by given pk
    fn delete(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<()> {
        let key = fmt_key::<Self>(pk.as_ref());

        conn.del(key)
    }
}

impl<T: RedisModel> HashModel for T {}
