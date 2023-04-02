#[cfg(feature = "aio")]
mod r#async;
#[cfg(not(feature = "aio"))]
mod sync;

#[cfg(feature = "aio")]
pub use r#async::HashModel;
#[cfg(not(feature = "aio"))]
pub use sync::HashModel;

mod cmds {
    use crate::redis_model::RedisModel;
    use redis::{Cmd, RedisResult, ToRedisArgs};

    /// Save data into redis database
    pub fn save<D: ToRedisArgs>(key: String, data: &D) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("HSET");
        cmd.arg(key).arg(data);

        Ok(cmd)
    }

    /// Get a list of all primary keys for current type
    pub fn get<M: RedisModel>(pk: impl AsRef<str>) -> RedisResult<Cmd> {
        let pk = pk.as_ref();

        let mut cmd = redis::cmd("HGETALL");

        if M::_is_pk_fmt(pk) {
            cmd.arg(pk);
        } else {
            cmd.arg(M::_fmt_pk(pk));
        }

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

        let mut cmd = redis::cmd("DEL");

        if M::_is_pk_fmt(pk) {
            cmd.arg(pk);
        } else {
            cmd.arg(M::_fmt_pk(pk));
        }

        Ok(cmd)
    }
}
