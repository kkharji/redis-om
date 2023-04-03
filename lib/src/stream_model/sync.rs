use redis::streams::StreamMaxlen;
use redis::ConnectionLike;
use redis::{FromRedisValue, RedisResult, ToRedisArgs};

use super::cmds;
use super::message::Message;
use super::reply::StreamReadReply;
use super::transformers;

impl Message {
    pub fn ack<Data: StreamModel, C: ConnectionLike>(&self, conn: &mut C) -> RedisResult<()> {
        Data::ack(&self.group, &[&self.id], conn)
    }
}

/// Stream Model for consuming and subscribing to redis stream data type
pub trait StreamModel: Sized {
    /// Data that will published and consumed from the stream
    type Data: ToRedisArgs + FromRedisValue;

    /// Redis Stream Key
    fn stream_key() -> &'static str;

    /// Group Name
    fn group_name(&self) -> &str;

    /// Consumer Name
    fn consumer_name(&self) -> &str;

    /// Publish self to stream, returning event id
    fn publish<C: ConnectionLike>(data: &Self::Data, conn: &mut C) -> RedisResult<String> {
        cmds::publish::<Self, _>(data)?.query(conn)
    }

    /// Ensure group stream exists for [`Self::stream_key`], creates a new if it doesn't exists.
    /// Errors if it fails to ensure stream
    fn ensure_group_stream<C: ConnectionLike>(&self, conn: &mut C) -> RedisResult<()> {
        let res = cmds::ensure_group_stream::<Self>(self)?.query(conn);
        transformers::ensure_group_stream_success(res)
    }

    /// Read from [`Self::stream_key`] with group name and consumer name.
    fn read<C: ConnectionLike>(
        &self,
        read_count: Option<usize>,
        block_interval: Option<usize>,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::read::<Self>(self, read_count, block_interval)?
            .query::<StreamReadReply>(conn)
            .map(|reply| transformers::stream_read_reply_to_messages(self, reply))?
    }

    /// Abstraction with default options and without a group.
    fn read_no_group<C: ConnectionLike>(
        id: impl AsRef<str>,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::read_no_group::<Self>(id)?
            .query::<StreamReadReply>(conn)
            .map(transformers::stream_read_no_group_reply_to_messages)?
    }

    /// Autoclaim an event and return a stream of messages found during the autoclaim.
    fn autoclaim<C: ConnectionLike>(
        group: impl AsRef<str>,
        consumer: impl AsRef<str>,
        min_idle_time: usize,
        last_autocalim_id: impl AsRef<str>,
        read_count: Option<usize>,
        conn: &mut C,
    ) -> RedisResult<(String, Vec<Message>)> {
        let group = group.as_ref();
        cmds::autoclaim::<Self>(
            group,
            consumer,
            min_idle_time,
            last_autocalim_id,
            read_count,
        )?
        .query(conn)
        .map(|(new_id, reply)| {
            transformers::autoclaim_range_to_id_and_messages(group, new_id, reply)
        })?
    }

    /// Acknowledge a given list of ids for group
    fn ack<'a, C: ConnectionLike>(
        group: impl ToRedisArgs,
        ids: &'a [impl ToRedisArgs],
        conn: &mut C,
    ) -> RedisResult<()> {
        cmds::ack::<Self>(group, ids)?.query(conn)
    }

    /// Return the length of the stream
    fn len<C: ConnectionLike>(conn: &mut C) -> RedisResult<usize> {
        cmds::len::<Self>()?.query(conn)
    }

    /// Trim a stream to a MAXLEN count.
    fn trim<C: ConnectionLike>(maxlen: StreamMaxlen, conn: &mut C) -> RedisResult<()> {
        cmds::trim::<Self>(maxlen)?.query(conn)
    }

    /// Returns a range of messages.
    ///
    /// Set `start` to `-` to begin at the first message.
    /// Set `end` to `+` to end the most recent message.
    ///
    /// You can pass message `id` to both `start` and `end`.
    ///
    fn range_count<C: ConnectionLike, S: ToRedisArgs, E: ToRedisArgs, N: ToRedisArgs>(
        start: S,
        end: E,
        count: N,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::range_count::<Self, _, _, _>(start, end, count)?
            .query(conn)
            .map(transformers::stream_range_to_messages)?
    }

    /// A method for paginating the stream
    fn range<C: ConnectionLike, S: ToRedisArgs, E: ToRedisArgs>(
        start: S,
        end: E,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::range::<Self, _, _>(start, end)?
            .query(conn)
            .map(transformers::stream_range_to_messages)?
    }

    /// A helper method for automatically returning all messages in a stream by `key`.
    /// **Use with caution!**
    fn range_all<C: ConnectionLike>(conn: &mut C) -> RedisResult<Vec<Message>> {
        cmds::range_all::<Self>()?
            .query(conn)
            .map(transformers::stream_range_to_messages)?
    }
}
