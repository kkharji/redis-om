use redis::aio::ConnectionLike;
use redis::streams::StreamMaxlen;
use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use std::future::Future;

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
    fn publish<C>(
        data: &Self::Data,
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<String>> + Send
    where
        C: ConnectionLike + Send,
    {
        async move { cmds::publish::<Self, _>(data)?.query_async(conn).await }
    }

    /// Ensure group stream exists for [`Self::stream_key`], creates a new if it doesn't exists.
    /// Errors if it fails to ensure stream
    fn ensure_group_stream<C>(&self, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send + Sync,
    {
        async move {
            let res = cmds::ensure_group_stream::<Self>(self)?
                .query_async(conn)
                .await;
            transformers::ensure_group_stream_success(res)
        }
    }

    /// Read from [`Self::stream_key`] with group name and consumer name.
    fn read<C>(
        &self,
        read_count: Option<usize>,
        block_interval: Option<usize>,
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<Vec<Message>>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send + Sync,
    {
        async move {
            cmds::read::<Self>(self, read_count, block_interval)?
                .query_async::<StreamReadReply>(conn)
                .await
                .map(|reply| transformers::stream_read_reply_to_messages(self, reply))?
        }
    }

    /// Abstraction with default options and without a group.
    fn read_no_group<C, Id>(
        id: Id,
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<Vec<Message>>> + Send
    where
        C: ConnectionLike + Send,
        Id: AsRef<str> + Send,
        Self: Send,
    {
        async move {
            cmds::read_no_group::<Self>(id)?
                .query_async::<StreamReadReply>(conn)
                .await
                .map(transformers::stream_read_no_group_reply_to_messages)?
        }
    }

    /// Autoclaim an event and return a stream of messages found during the autoclaim.
    fn autoclaim<C, G, CN, L>(
        group: G,
        consumer: CN,
        min_idle_time: usize,
        last_autocalim_id: L,
        read_count: Option<usize>,
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<(String, Vec<Message>)>> + Send
    where
        C: ConnectionLike + Send,
        G: AsRef<str> + Send,
        CN: AsRef<str> + Send,
        L: AsRef<str> + Send,
        Self: Send,
    {
        async move {
            let group_ref = group.as_ref();
            cmds::autoclaim::<Self>(
                group_ref,
                consumer,
                min_idle_time,
                last_autocalim_id,
                read_count,
            )?
            .query_async(conn)
            .await
            .map(|(new_id, reply)| {
                transformers::autoclaim_range_to_id_and_messages(group_ref, new_id, reply)
            })?
        }
    }

    /// Acknowledge a given list of ids for group
    fn ack<C, I, G>(
        group: G,
        ids: &[I],
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        I: ToRedisArgs + Sync,
        G: ToRedisArgs + Send,
        Self: Send,
    {
        async move {
            let cmd = cmds::ack::<Self>(group, ids)?;
            cmd.query_async(conn).await
        }
    }

    /// Return the length of the stream
    fn len<C>(conn: &mut C) -> impl Future<Output = RedisResult<usize>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send,
    {
        async move { cmds::len::<Self>()?.query_async(conn).await }
    }

    /// Trim a stream to a MAXLEN count.
    fn trim<C>(maxlen: StreamMaxlen, conn: &mut C) -> impl Future<Output = RedisResult<()>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send,
    {
        async move { cmds::trim::<Self>(maxlen)?.query_async(conn).await }
    }

    /// Returns a range of messages.
    ///
    /// Set `start` to `-` to begin at the first message.
    /// Set `end` to `+` to end the most recent message.
    ///
    /// You can pass message `id` to both `start` and `end`.
    ///
    fn range_count<C, S, E, N>(
        start: S,
        end: E,
        count: N,
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<Vec<Message>>> + Send
    where
        C: ConnectionLike + Send,
        S: ToRedisArgs + Send,
        E: ToRedisArgs + Send,
        N: ToRedisArgs + Send,
        Self: Send,
    {
        async move {
            cmds::range_count::<Self, _, _, _>(start, end, count)?
                .query_async(conn)
                .await
                .map(transformers::stream_range_to_messages)?
        }
    }

    /// A method for paginating the stream
    fn range<C, S, E>(
        start: S,
        end: E,
        conn: &mut C,
    ) -> impl Future<Output = RedisResult<Vec<Message>>> + Send
    where
        C: ConnectionLike + Send,
        S: ToRedisArgs + Send,
        E: ToRedisArgs + Send,
        Self: Send,
    {
        async move {
            cmds::range::<Self, _, _>(start, end)?
                .query_async(conn)
                .await
                .map(transformers::stream_range_to_messages)?
        }
    }

    /// A helper method for automatically returning all messages in a stream by `key`.
    /// **Use with caution!**
    fn range_all<C>(conn: &mut C) -> impl Future<Output = RedisResult<Vec<Message>>> + Send
    where
        C: ConnectionLike + Send,
        Self: Send,
    {
        async move {
            cmds::range_all::<Self>()?
                .query_async(conn)
                .await
                .map(transformers::stream_range_to_messages)?
        }
    }
}
