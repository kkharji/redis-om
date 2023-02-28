use super::cmds;
use crate::{RedisModel, RedisSearchModel};
use redis::ConnectionLike;
use redis::{FromRedisValue, RedisResult, ToRedisArgs};

/// Hash Object Model
pub trait HashModel: RedisModel + RedisSearchModel + ToRedisArgs + FromRedisValue {
    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn redis_prefix() -> &'static str {
        <Self as RedisModel>::_prefix_key()
    }

    /// Redis search schema
    fn redissearch_schema() -> &'static str {
        <Self as RedisSearchModel>::_REDIS_SEARCH_SCHEMA
    }

    /// Save Self into redis database
    fn save<C: ConnectionLike>(&mut self, conn: &mut C) -> RedisResult<()> {
        self._ensure_pk();
        let key = self._get_redis_key();
        cmds::save(key, self)?.query(conn)
    }

    /// Get a list of all primary keys for current type
    fn all_pks<C: ConnectionLike>(conn: &mut C) -> RedisResult<redis::Iter<'_, String>> {
        cmds::all_pks::<Self>()?.iter(conn)
    }

    /// Get a list of all primary keys for current type
    fn get<C: ConnectionLike>(pk: impl AsRef<str>, conn: &mut C) -> RedisResult<Self> {
        cmds::get::<Self>(pk)?.query(conn)
    }

    /// Delete by given pk
    fn delete<C: ConnectionLike>(pk: impl AsRef<str>, conn: &mut C) -> RedisResult<()> {
        cmds::delete::<Self>(pk)?.query(conn)
    }

    /// Expire Self at given duration
    fn expire<C: ConnectionLike>(&self, secs: usize, conn: &mut C) -> RedisResult<()> {
        self._expire_cmd(secs)?.query(conn)
    }
}
