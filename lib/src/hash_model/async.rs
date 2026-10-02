use super::cmds;
use crate::{RedisModel, RedisSearchModel};
use redis::aio::ConnectionLike;
use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use std::future::Future;

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
    fn save<C>(&mut self, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send,
    {
        async move {
            self._ensure_pk();
            let key = self._get_redis_key();
            let cmd = cmds::save(key, self)?;
            cmd.query_async(conn).await
        }
    }

    /// Get a list of all primary keys for current type
    fn all_pks<'a, C>(
        conn: &'a mut C,
    ) -> impl Future<Output = RedisResult<redis::AsyncIter<'a, String>>> + Send + 'a
    where
        C: ConnectionLike + Send + 'a,
    {
        async move { cmds::all_pks::<Self>()?.iter_async(conn).await }
    }

    /// Get a list of all primary keys for current type
    fn get<'a, C, S>(pk: S, conn: &'a mut C) -> impl Future<Output = RedisResult<Self>> + Send + 'a
    where
        S: AsRef<str> + Send + 'a,
        C: ConnectionLike + Send + 'a,
    {
        async move { cmds::get::<Self>(pk)?.query_async(conn).await }
    }

    /// Delete by given pk
    fn delete<'a, S, C>(pk: S, conn: &'a mut C) -> impl Future<Output = RedisResult<()>> + Send + 'a
    where
        S: AsRef<str> + Send + 'a,
        C: ConnectionLike + Send + 'a,
    {
        async move { cmds::delete::<Self>(pk)?.query_async(conn).await }
    }

    /// Expire Self at given duration
    fn expire<C>(&self, secs: usize, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send + Sync,
    {
        async move { self._expire_cmd(secs)?.query_async(conn).await }
    }
}
