use super::cmds;
use crate::{RedisModel, RedisSearchModel};
use redis::aio::ConnectionLike;
use redis::{FromRedisValue, RedisResult, ToRedisArgs};

/// Hash Object Model
#[async_trait::async_trait]
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
    async fn save<C>(&mut self, conn: &mut C) -> RedisResult<()>
    where
        C: ConnectionLike + Send,
    {
        self._ensure_pk();
        let key = self._get_redis_key();
        let cmd = cmds::save(key, self)?;
        cmd.query_async(conn).await
    }

    /// Get a list of all primary keys for current type
    async fn all_pks<C: ConnectionLike + Send>(
        conn: &mut C,
    ) -> RedisResult<redis::AsyncIter<'_, String>> {
        cmds::all_pks::<Self>()?.iter_async(conn).await
    }

    /// Get a list of all primary keys for current type
    async fn get<C, S>(pk: S, conn: &mut C) -> RedisResult<Self>
    where
        S: AsRef<str> + Send,
        C: ConnectionLike + Send,
    {
        cmds::get::<Self>(pk)?.query_async(conn).await
    }

    /// Delete by given pk
    async fn delete<S, C>(pk: S, conn: &mut C) -> RedisResult<()>
    where
        S: AsRef<str> + Send,
        C: ConnectionLike + Send,
    {
        cmds::delete::<Self>(pk)?.query_async(conn).await
    }

    /// Expire Self at given duration
    async fn expire<C>(&self, secs: usize, conn: &mut C) -> RedisResult<()>
    where
        C: ConnectionLike + Send,
    {
        self._expire_cmd(secs)?.query_async(conn).await
    }
}
