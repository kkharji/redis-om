#[cfg(feature = "aio")]
mod r#async;
#[cfg(not(feature = "aio"))]
mod sync;

#[cfg(feature = "aio")]
pub use r#async::JsonModel;
#[cfg(not(feature = "aio"))]
pub use sync::JsonModel;

use redis::{ErrorKind, RedisError, RedisResult};

mod cmds {
    use crate::redis_model::RedisModel;
    use redis::{Cmd, RedisResult};

    /// Save data into redis database
    pub fn save<D: serde::Serialize>(key: String, data: &D) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("JSON.SET");

        cmd.arg(key).arg("$").arg(serde_json::to_string(data)?);

        Ok(cmd)
    }

    /// Get a list of all primary keys for current type
    pub fn get<M: RedisModel>(pk: impl AsRef<str>) -> RedisResult<Cmd> {
        let pk = pk.as_ref();

        let mut cmd = redis::cmd("JSON.GET");

        if M::_is_pk_fmt(pk) {
            cmd.arg(pk);
        } else {
            cmd.arg(M::_fmt_pk(pk));
        }

        cmd.arg("$");

        Ok(cmd)
    }

    /// Get a list of all primary keys for current type
    pub fn all_pks<M: RedisModel>() -> RedisResult<Cmd> {
        let pattern = format!("{}:*", M::_prefix_key());

        let mut cmd = redis::cmd("SCAN");
        cmd.cursor_arg(0).arg("MATCH").arg(pattern);

        Ok(cmd)
    }

    /// Delete by given pk
    pub fn delete<M: RedisModel>(pk: impl AsRef<str>) -> RedisResult<Cmd> {
        let pk = pk.as_ref();

        let mut cmd = redis::cmd("JSON.DEL");

        if M::_is_pk_fmt(pk) {
            cmd.arg(pk);
        } else {
            cmd.arg(M::_fmt_pk(pk));
        }

        cmd.arg("$");

        Ok(cmd)
    }
}

fn parse_from_get_resp<D: for<'de> serde::Deserialize<'de>>(resp: String) -> RedisResult<D> {
    let value = serde_json::from_str::<'_, serde_json::Value>(&resp)
        .map_err(RedisError::from)?
        .as_array()
        .and_then(|f| f.first())
        .map(|v| v.to_owned())
        .ok_or_else(|| {
            RedisError::from((
                ErrorKind::Serialize,
                "expect an array with at least one item",
                format!("got {:#?}", resp),
            ))
        })?;

    serde_json::from_value(value).map_err(|e| e.into())
}
