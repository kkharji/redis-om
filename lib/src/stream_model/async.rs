use redis::aio::ConnectionLike;
use redis::streams::StreamMaxlen;
use redis::{FromRedisValue, RedisResult, ToRedisArgs};

use super::cmds;
use super::message::Message;
use super::reply::StreamReadReply;
use super::transformers;

impl Message {
    pub async fn ack<Data: StreamModel, C: ConnectionLike + Send>(
        &self,
        conn: &mut C,
    ) -> RedisResult<()> {
        Data::ack(&self.group, &[&self.id], conn).await
    }
}

/// Stream Model for consuming and subscribing to redis stream data type
#[async_trait::async_trait]
pub trait StreamModel: Sized + Send {
    /// Data that will published and consumed from the stream
    type Data: ToRedisArgs + FromRedisValue + Sync;

    /// Redis Stream Key
    fn stream_key() -> &'static str;

    /// Group Name
    fn group_name(&self) -> &str;

    /// Consumer Name
    fn consumer_name(&self) -> &str;

    /// Publish self to stream, returning event id
    async fn publish<C: ConnectionLike + Send>(
        data: &Self::Data,
        conn: &mut C,
    ) -> RedisResult<String> {
        cmds::publish::<Self, _>(data)?.query_async(conn).await
    }

    /// Ensure group stream exists for [`Self::stream_key`], creates a new if it doesn't exists.
    /// Errors if it fails to ensure stream
    async fn ensure_group_stream<C: ConnectionLike + Send>(&self, conn: &mut C) -> RedisResult<()> {
        let res = cmds::ensure_group_stream::<Self>(self)?
            .query_async(conn)
            .await;
        transformers::ensure_group_stream_success(res)
    }

    /// Read from [`Self::stream_key`] with group name and consumer name.
    async fn read<C: ConnectionLike + Send>(
        &self,
        read_count: Option<usize>,
        block_interval: Option<usize>,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::read::<Self>(self, read_count, block_interval)?
            .query_async::<_, StreamReadReply>(conn)
            .await
            .map(|reply| transformers::stream_read_reply_to_messages(self, reply))?
    }

    /// Abstraction with default options and without a group.
    async fn read_no_group<C: ConnectionLike + Send>(
        id: impl AsRef<str> + Send,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::read_no_group::<Self>(id)?
            .query_async::<_, StreamReadReply>(conn)
            .await
            .map(transformers::stream_read_no_group_reply_to_messages)?
    }

    /// Autoclaim an event and return a stream of messages found during the autoclaim.
    async fn autoclaim<C: ConnectionLike + Send>(
        group: impl AsRef<str> + Send,
        consumer: impl AsRef<str> + Send,
        min_idle_time: usize,
        last_autocalim_id: impl AsRef<str> + Send,
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
        .query_async(conn)
        .await
        .map(|(new_id, reply)| {
            transformers::autoclaim_range_to_id_and_messages(group, new_id, reply)
        })?
    }

    /// Acknowledge a given list of ids for group
    async fn ack<C: ConnectionLike + Send, I: ToRedisArgs + Sync>(
        group: impl ToRedisArgs + Send,
        ids: &[I],
        conn: &mut C,
    ) -> RedisResult<()> {
        let cmd = cmds::ack::<Self>(group, ids)?;

        cmd.query_async(conn).await
    }

    /// Return the length of the stream
    async fn len<C: ConnectionLike + Send>(conn: &mut C) -> RedisResult<usize> {
        cmds::len::<Self>()?.query_async(conn).await
    }

    /// Trim a stream to a MAXLEN count.
    async fn trim<C: ConnectionLike + Send>(maxlen: StreamMaxlen, conn: &mut C) -> RedisResult<()> {
        cmds::trim::<Self>(maxlen)?.query_async(conn).await
    }

    /// Returns a range of messages.
    ///
    /// Set `start` to `-` to begin at the first message.
    /// Set `end` to `+` to end the most recent message.
    ///
    /// You can pass message `id` to both `start` and `end`.
    ///
    async fn range_count<
        C: ConnectionLike + Send,
        S: ToRedisArgs + Send,
        E: ToRedisArgs + Send,
        N: ToRedisArgs + Send,
    >(
        start: S,
        end: E,
        count: N,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::range_count::<Self, _, _, _>(start, end, count)?
            .query_async(conn)
            .await
            .map(transformers::stream_range_to_messages)?
    }

    /// A method for paginating the stream
    async fn range<C: ConnectionLike + Send, S: ToRedisArgs + Send, E: ToRedisArgs + Send>(
        start: S,
        end: E,
        conn: &mut C,
    ) -> RedisResult<Vec<Message>> {
        cmds::range::<Self, _, _>(start, end)?
            .query_async(conn)
            .await
            .map(transformers::stream_range_to_messages)?
    }

    /// A helper method for automatically returning all messages in a stream by `key`.
    /// **Use with caution!**
    async fn range_all<C: ConnectionLike + Send>(conn: &mut C) -> RedisResult<Vec<Message>> {
        cmds::range_all::<Self>()?
            .query_async(conn)
            .await
            .map(transformers::stream_range_to_messages)?
    }
}
