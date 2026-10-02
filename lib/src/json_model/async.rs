use redis::{AsyncIter, RedisResult, aio::ConnectionLike};
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;

use super::{cmds, parse_from_get_resp};
use crate::{RedisModel, RedisSearchModel};

/// Hash Object Model
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
    fn save<C>(&mut self, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send,
    {
        async move {
            self._ensure_pk();
            let cmd = cmds::save(self._get_redis_key(), self)?;
            cmd.query_async(conn).await
        }
    }

    /// Get a list of all primary keys for current type
    fn all_pks<'a, C>(
        conn: &'a mut C,
    ) -> impl Future<Output = RedisResult<AsyncIter<'a, String>>> + Send + 'a
    where
        C: ConnectionLike + Send + 'a,
        Self: Send,
    {
        async move {
            let cmd = cmds::all_pks::<Self>()?;
            cmd.iter_async(conn).await
        }
    }

    /// Get a specific record by primary key
    fn get<'a, S, C>(pk: S, conn: &'a mut C) -> impl Future<Output = RedisResult<Self>> + Send + 'a
    where
        S: AsRef<str> + Send + 'a,
        C: ConnectionLike + Send + 'a,
        Self: Send,
    {
        async move {
            let pk = pk.as_ref();
            let cmd = cmds::get::<Self>(pk)?;
            let resp = cmd.query_async(conn).await?;
            parse_from_get_resp(resp)
        }
    }

    /// Delete by given pk
    fn delete<S, C>(pk: S, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        S: AsRef<str> + Send,
        C: ConnectionLike + Send,
        Self: Send,
    {
        async move {
            let cmd = cmds::delete::<Self>(pk)?;
            cmd.query_async(conn).await
        }
    }

    /// Expire Self at given duration
    fn expire<C>(&self, secs: usize, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send + Sync,
    {
        async move {
            let cmd = self._expire_cmd(secs)?;
            cmd.query_async(conn).await
        }
    }
}
