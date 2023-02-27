use crate::redis_model::RedisModel;
use crate::shared::{Commands, Conn};
use crate::RedisSearchModel;
use ::redis::RedisResult;
use redis::{ErrorKind, JsonCommands, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Hash Object Model
pub trait JsonModel: RedisModel + RedisSearchModel + Serialize + DeserializeOwned {
    /// Get Redis key to be used in storing HashModel object.
    /// This should by default that HashModel name in lowercase.
    fn redis_prefix() -> &'static str {
        <Self as RedisModel>::_prefix_key()
    }

    /// Save Self into redis database
    fn save(&mut self, conn: &mut Conn) -> RedisResult<()> {
        self._ensure_pk();

        conn.json_set(self._get_redis_key(), "$", &self)
    }

    /// Get a list of all primary keys for current type
    fn all_pks(conn: &mut Conn) -> RedisResult<redis::Iter<'_, String>> {
        let pattern = format!("{}:*", Self::redis_prefix());

        conn.scan_match(pattern)
    }

    /// Get a list of all primary keys for current type
    fn get(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<Self> {
        let pk = pk.as_ref();
        let res: String = if Self::_is_pk_fmt(pk) {
            conn.json_get(pk, "$")
        } else {
            conn.json_get(Self::_fmt_pk(pk), "$")
        }?;

        let value = serde_json::from_str::<'_, serde_json::Value>(&res)
            .map_err(RedisError::from)?
            .as_array()
            .and_then(|f| f.first())
            .map(|v| v.to_owned())
            .ok_or_else(|| {
                RedisError::from((
                    ErrorKind::Serialize,
                    "expect an array with at least one item",
                    format!("got {:#?}", res),
                ))
            })?;

        serde_json::from_value(value).map_err(|e| e.into())
    }

    /// Delete by given pk
    fn delete(pk: impl AsRef<str>, conn: &mut Conn) -> RedisResult<()> {
        let pk = pk.as_ref();
        if Self::_is_pk_fmt(pk) {
            conn.json_del(pk, "$")
        } else {
            conn.json_del(Self::_fmt_pk(pk), "$")
        }
    }

    /// Redis search schema
    fn redissearch_schema() -> &'static str {
        <Self as RedisSearchModel>::_REDIS_SEARCH_SCHEMA
    }

    /// Expire Self at given duration
    fn expire(&self, secs: usize, conn: &mut Conn) -> RedisResult<()> {
        self._expire(secs, conn)
    }
}
