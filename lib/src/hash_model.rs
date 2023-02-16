use crate::redis_model::RedisModel;
use crate::shared::{Commands, Conn};
use ::redis::RedisResult;
/// Hash Object Model
pub trait HashModel: RedisModel {
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
        conn.hgetall(Self::_fmt_pk(pk.as_ref()))
    }

    /// Delete by given pk
    fn delete(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<()> {
        conn.del(Self::_fmt_pk(pk.as_ref()))
    }
}

impl<T: RedisModel> HashModel for T {}
