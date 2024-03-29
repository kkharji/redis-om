use super::reply::StreamId;
use redis::{FromRedisValue, RedisResult, Value};

pub struct Message {
    /// The ID of this message (generated by Redis).
    pub id: String,
    /// The group this message belongs to.
    pub group: String,
    /// The value of message
    value: Value,
}

impl Message {
    pub fn new(value: StreamId, group: String) -> Self {
        Self {
            id: value.id,
            group,
            value: value.value,
        }
    }

    pub fn data<Data: FromRedisValue>(&self) -> RedisResult<Data> {
        Data::from_redis_value(&self.value)
    }
}
