mod message;
mod reply;

#[cfg(feature = "aio")]
mod r#async;
#[cfg(not(feature = "aio"))]
mod sync;

#[cfg(feature = "aio")]
pub use r#async::StreamModel;
#[cfg(not(feature = "aio"))]
pub use sync::StreamModel;

mod cmds {
    use redis::{
        streams::{StreamMaxlen, StreamReadOptions},
        Cmd, RedisResult, ToRedisArgs,
    };

    use super::StreamModel;

    pub fn publish<S: StreamModel, D: ToRedisArgs>(data: &D) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XADD");
        cmd.arg(S::stream_key()).arg("*").arg(data);

        Ok(cmd)
    }

    pub fn ensure_group_stream<S: StreamModel>(s: &impl StreamModel) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XGROUP");
        cmd.arg("CREATE")
            .arg(S::stream_key())
            .arg(s.group_name())
            .arg("$");

        Ok(cmd)
    }

    pub fn read<S: StreamModel>(
        s: &impl StreamModel,
        read_count: Option<usize>,
        block_interval: Option<usize>,
    ) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XREADGROUP");
        let mut opts = StreamReadOptions::default().group(s.group_name(), s.consumer_name());
        if let Some(read_count) = read_count {
            opts = opts.count(read_count)
        }
        if let Some(block_interval) = block_interval {
            opts = opts.block(block_interval)
        }

        cmd.arg(opts)
            .arg("STREAMS")
            .arg(&[S::stream_key()])
            .arg(&[">"]);

        Ok(cmd)
    }

    pub fn read_no_group<S: StreamModel>(id: impl AsRef<str>) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XREAD");

        cmd.arg("STREAMS")
            .arg(&[S::stream_key()])
            .arg(&[id.as_ref()]);

        Ok(cmd)
    }

    pub fn autoclaim<S: StreamModel>(
        group: &str,
        consumer: impl AsRef<str>,
        min_idle_time: usize,
        last_autocalim_id: impl AsRef<str>,
        read_count: Option<usize>,
    ) -> RedisResult<Cmd> {
        let id = last_autocalim_id.as_ref();
        let mut cmd = redis::cmd("XAUTOCLAIM");

        cmd.arg(S::stream_key())
            .arg(group)
            .arg(consumer.as_ref())
            .arg(min_idle_time)
            .arg(&id);

        if let Some(read_count) = read_count {
            cmd.arg("COUNT").arg(read_count);
        }

        Ok(cmd)
    }

    pub fn ack<S: StreamModel>(
        group: impl ToRedisArgs,
        ids: &[impl ToRedisArgs],
    ) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XACK");
        cmd.arg(S::stream_key()).arg(group).arg(ids);
        Ok(cmd)
    }

    pub fn trim<S: StreamModel>(maxlen: StreamMaxlen) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XTRIM");
        cmd.arg(S::stream_key()).arg(maxlen);
        Ok(cmd)
    }

    pub fn len<S: StreamModel>() -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XLEN");
        cmd.arg(S::stream_key());
        Ok(cmd)
    }

    pub fn range_count<S: StreamModel, B: ToRedisArgs, E: ToRedisArgs, C: ToRedisArgs>(
        start: B,
        end: E,
        count: C,
    ) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XRANGE");

        cmd.arg(S::stream_key())
            .arg(start)
            .arg(end)
            .arg("COUNT")
            .arg(count);

        Ok(cmd)
    }

    pub fn range<S: StreamModel, B: ToRedisArgs, E: ToRedisArgs>(
        start: B,
        end: E,
    ) -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XRANGE");

        cmd.arg(S::stream_key()).arg(start).arg(end);

        Ok(cmd)
    }

    pub fn range_all<S: StreamModel>() -> RedisResult<Cmd> {
        let mut cmd = redis::cmd("XREVRANGE");

        cmd.arg(S::stream_key()).arg("+").arg("-");

        Ok(cmd)
    }
}

mod transformers {
    use super::{
        message::Message,
        reply::{StreamRangeReply, StreamReadReply},
        StreamModel,
    };
    use redis::RedisResult;
    use tap::Pipe;

    pub fn stream_read_reply_to_messages(
        s: &impl StreamModel,
        reply: StreamReadReply,
    ) -> RedisResult<Vec<Message>> {
        reply
            .keys
            .into_iter()
            .flat_map(|stream| {
                let into_iter = stream.ids.into_iter();
                into_iter.map(|item| Message::new(item, s.group_name().to_string()))
            })
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn stream_read_no_group_reply_to_messages(
        reply: StreamReadReply,
    ) -> RedisResult<Vec<Message>> {
        reply
            .keys
            .into_iter()
            .flat_map(|stream| {
                let into_iter = stream.ids.into_iter();
                into_iter.map(|item| Message::new(item, "none".into()))
            })
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn stream_range_to_messages(reply: StreamRangeReply) -> RedisResult<Vec<Message>> {
        reply
            .ids
            .into_iter()
            .map(move |item| Message::new(item, "none".into()))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn ensure_group_stream_success(res: RedisResult<String>) -> RedisResult<()> {
        // It is expected behavior that this will fail when already initalized
        // Expected error: `BUSYGROUP: Consumer Group name already exists`
        if let Err(err) = res.map(|_: String| ()) {
            if err.code() != Some("BUSYGROUP") {
                return Err(err);
            }
        }
        Ok(())
    }

    pub fn autoclaim_range_to_id_and_messages(
        group: &str,
        new_id: String,
        reply: StreamRangeReply,
    ) -> RedisResult<(String, Vec<Message>)> {
        let resp = reply
            .ids
            .into_iter()
            .map(move |item| Message::new(item, group.to_owned()))
            .collect::<Vec<_>>();

        Ok((new_id, resp))
    }
}
