use super::{cmds, parse_from_get_resp};
use crate::{RedisModel, RedisSearchModel};
use redis::{aio::ConnectionLike, AsyncIter, RedisResult};
use serde::{de::DeserializeOwned, Serialize};

/// Hash Object Model
#[async_trait::async_trait]
pub trait JsonModel: RedisModel + RedisSearchModel + Serialize + DeserializeOwned {
    /// Redis search schema
    fn redissearch_schema() -> &'static str {
        <Self as RedisSearchModel>::_REDIS_SEARCH_SCHEMA
    }

    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn redis_prefix() -> &'static str {
        Self::_prefix_key()
    }

    /// Save Self into redis database
    async fn save<C>(&mut self, conn: &mut C) -> RedisResult<()>
    where
        C: ConnectionLike + Send,
    {
        self._ensure_pk();
        let cmd = cmds::save(self._get_redis_key(), self)?;

        cmd.query_async(conn).await
    }

    /// Get a list of all primary keys for current type
    async fn all_pks<C>(conn: &mut C) -> RedisResult<AsyncIter<'_, String>>
    where
        C: ConnectionLike + Send,
    {
        let cmd = cmds::all_pks::<Self>()?;

        cmd.iter_async(conn).await
    }

    /// Get a list of all primary keys for current type
    async fn get<S, C>(pk: S, conn: &mut C) -> RedisResult<Self>
    where
        S: AsRef<str> + Send,
        C: ConnectionLike + Send,
    {
        let pk = pk.as_ref();
        let cmd = cmds::get::<Self>(pk)?;
        let resp = cmd.query_async(conn).await?;

        parse_from_get_resp(resp)
    }

    /// Delete by given pk
    async fn delete<S, C>(pk: S, conn: &mut C) -> RedisResult<()>
    where
        S: AsRef<str> + Send,
        C: ConnectionLike + Send,
    {
        let cmd = cmds::delete::<Self>(pk)?;

        cmd.query_async(conn).await
    }

    /// Expire Self at given duration
    async fn expire<C>(&self, secs: usize, conn: &mut C) -> RedisResult<()>
    where
        C: ConnectionLike + Send,
    {
        let cmd = self._expire_cmd(secs)?;

        cmd.query_async(conn).await
    }
}
