mod message;
mod reply;

use crate::shared::Conn;
use redis::streams::{StreamMaxlen, StreamReadOptions};
use redis::{Commands, FromRedisValue, RedisResult, ToRedisArgs};
use tap::Pipe;

use message::Message;
use reply::StreamReadReply;

use self::reply::StreamRangeReply;

/// Stream Model for consuming and subscribing to redis stream data type
pub trait StreamModel {
    /// Data that will published and consumed from the stream
    type Data: ToRedisArgs + FromRedisValue;

    /// Redis Stream Key
    fn stream_key() -> &'static str;

    /// Group Name
    fn group_name(&self) -> &str;

    /// Consumer Name
    fn consumer_name(&self) -> &str;

    /// Publish self to stream, returning event id
    fn publish(data: &Self::Data, conn: &mut Conn) -> RedisResult<String> {
        conn.xadd_map(Self::stream_key(), "*", data)
    }

    /// Ensure group stream exists for [`Self::stream_key`], creates a new if it doesn't exists.
    /// Errors if it fails to ensure stream
    fn ensure_group_stream(&self, conn: &mut Conn) -> RedisResult<()> {
        if let Err(err) = conn
            .xgroup_create_mkstream(Self::stream_key(), self.group_name(), "$")
            .map(|_: String| ())
        {
            // It is expected behavior that this will fail when already initalized
            // Expected error: `BUSYGROUP: Consumer Group name already exists`
            if err.code() != Some("BUSYGROUP") {
                return Err(err);
            }
        }
        Ok(())
    }

    /// Read from [`Self::stream_key`] with group name and consumer name.
    fn read(
        &self,
        read_count: Option<usize>,
        block_interval: Option<usize>,
        conn: &mut Conn,
    ) -> RedisResult<Vec<Message>> {
        let mut opts = StreamReadOptions::default().group(self.group_name(), self.consumer_name());
        if let Some(read_count) = read_count {
            opts = opts.count(read_count)
        }
        if let Some(block_interval) = block_interval {
            opts = opts.block(block_interval)
        }

        conn.xread_options::<_, _, StreamReadReply>(&[Self::stream_key()], &[">"], &opts)?
            .keys
            .into_iter()
            .flat_map(|stream| {
                let into_iter = stream.ids.into_iter();
                into_iter.map(|item| Message::new(item, self.group_name().to_string()))
            })
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    /// Abstraction with default options and without a group.
    fn read_no_group(id: impl AsRef<str>, conn: &mut Conn) -> RedisResult<Vec<Message>> {
        conn.xread::<_, _, StreamReadReply>(&[Self::stream_key()], &[id.as_ref()])?
            .keys
            .into_iter()
            .flat_map(|stream| {
                let into_iter = stream.ids.into_iter();
                into_iter.map(|item| Message::new(item, "none".into()))
            })
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    /// Autoclaim an event and return a stream of messages found during the autoclaim.
    fn autoclaim(
        group: impl AsRef<str>,
        consumer: impl AsRef<str>,
        min_idle_time: usize,
        last_autocalim_id: impl AsRef<str>,
        read_count: Option<usize>,
        conn: &mut Conn,
    ) -> RedisResult<(String, Vec<Message>)> {
        let id = last_autocalim_id.as_ref();
        let mut cmd = redis::cmd("XAUTOCLAIM");

        cmd.arg(Self::stream_key())
            .arg(group.as_ref())
            .arg(consumer.as_ref())
            .arg(min_idle_time)
            .arg(&id);

        if let Some(read_count) = read_count {
            cmd.arg("COUNT").arg(read_count);
        }

        let (new_id, StreamRangeReply { ids: res }) = cmd.query(conn)?;

        let resp = res
            .into_iter()
            .map(move |item| Message::new(item, group.as_ref().to_owned()))
            .collect::<Vec<_>>();

        Ok((new_id, resp))
    }

    /// Acknowledge a given list of ids for group
    fn ack<'a, I: ToRedisArgs>(
        ids: &'a [I],
        group: impl ToRedisArgs,
        conn: &mut Conn,
    ) -> RedisResult<()> {
        conn.xack(Self::stream_key(), group, ids)
    }

    /// Return the length of the stream
    fn len(conn: &mut Conn) -> RedisResult<usize> {
        conn.xlen(Self::stream_key())
    }

    /// Trim a stream to a MAXLEN count.
    fn trim(maxlen: StreamMaxlen, conn: &mut Conn) -> RedisResult<()> {
        conn.xtrim(Self::stream_key(), maxlen)
    }

    /// Returns a range of messages.
    ///
    /// Set `start` to `-` to begin at the first message.
    /// Set `end` to `+` to end the most recent message.
    ///
    /// You can pass message `id` to both `start` and `end`.
    ///
    fn range_count<S: ToRedisArgs, E: ToRedisArgs, C: ToRedisArgs>(
        start: S,
        end: E,
        count: C,
        conn: &mut Conn,
    ) -> RedisResult<Vec<Message>> {
        conn.xrange_count::<_, _, _, _, StreamRangeReply>(Self::stream_key(), start, end, count)?
            .ids
            .into_iter()
            .map(move |item| Message::new(item, "none".into()))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    /// A method for paginating the stream
    fn range<S: ToRedisArgs, E: ToRedisArgs>(
        start: S,
        end: E,
        conn: &mut Conn,
    ) -> RedisResult<Vec<Message>> {
        conn.xrange::<_, _, _, StreamRangeReply>(Self::stream_key(), start, end)?
            .ids
            .into_iter()
            .map(move |item| Message::new(item, "none".into()))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    /// A helper method for automatically returning all messages in a stream by `key`.
    /// **Use with caution!**
    fn range_all(conn: &mut Conn) -> RedisResult<Vec<Message>> {
        conn.xrange_all::<_, StreamRangeReply>(Self::stream_key())?
            .ids
            .into_iter()
            .map(move |item| Message::new(item, "none".into()))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }
}
