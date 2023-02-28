use std::collections::HashMap;

use redis::{FromRedisValue, RedisResult, Value};

/// Represents a stream `id` and its field/values as a `HashMap`
#[derive(Debug, Clone)]
pub struct StreamId {
    /// The stream `id` (entry ID) of this particular message.
    pub id: String,
    /// All fields in this message, associated with their respective values.
    pub value: Value,
}

#[derive(Default, Debug, Clone)]
pub struct StreamKey {
    /// The stream `key`.
    pub key: String,
    /// The parsed stream `id`'s.
    pub ids: Vec<StreamId>,
}

#[derive(Default, Debug, Clone)]
pub struct StreamReadReply {
    /// Complex data structure containing a payload for each key in this array
    pub keys: Vec<StreamKey>,
}

#[derive(Default, Debug, Clone)]
pub struct StreamRangeReply {
    /// Complex data structure containing a payload for each ID in this array
    pub ids: Vec<StreamId>,
}

impl FromRedisValue for StreamRangeReply {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let rows: Vec<HashMap<String, Value>> = redis::from_redis_value(v)?;
        let ids: Vec<StreamId> = rows
            .into_iter()
            .flat_map(|row| row.into_iter().map(|(id, value)| StreamId { id, value }))
            .collect();

        Ok(StreamRangeReply { ids })
    }
}

type SRRows = Vec<HashMap<String, Vec<HashMap<String, Value>>>>;
impl FromRedisValue for StreamReadReply {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let rows: SRRows = redis::from_redis_value(v)?;
        let keys = rows
            .into_iter()
            .flat_map(|row| {
                row.into_iter().map(|(key, entry)| {
                    let ids = entry
                        .into_iter()
                        .flat_map(|id_row| {
                            id_row.into_iter().map(|(id, value)| StreamId { id, value })
                        })
                        .collect();
                    StreamKey { key, ids }
                })
            })
            .collect();
        Ok(StreamReadReply { keys })
    }
}
