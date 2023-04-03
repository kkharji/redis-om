#![cfg(feature = "aio")]

use redis_om::{RedisTransportValue, StreamModel};
use std::error::Error;
use tokio::test;

type Result<T = (), E = Box<dyn Error>> = std::result::Result<T, E>;

#[derive(RedisTransportValue)]
pub enum RoomServiceKind {
    Clean,
    ExtraTowels,
    ExtraPillows,
    FoodOrder,
}

#[derive(StreamModel)]
#[redis(key = "test-events")]
pub struct RoomServiceEvent {
    status: String,
    room: usize,
    kind: RoomServiceKind,
}

fn client() -> Result<redis::Client> {
    Ok(redis::Client::open("redis://127.0.0.1/")?)
}

#[test]
async fn example() -> Result {
    let client = client()?;
    let mut conn = client.get_async_connection().await?;
    let manager = RoomServiceEventManager::new("Staff");

    manager.ensure_group_stream(&mut conn).await?;

    let event = RoomServiceEvent {
        status: "pending".into(),
        room: 3,
        kind: RoomServiceKind::Clean,
    };

    RoomServiceEventManager::publish(&event, &mut conn).await?;

    let read = manager.read(None, None, &mut conn).await?;
    let message = read.first().unwrap().data::<RoomServiceEvent>()?;

    assert_eq!(message.room, event.room);

    Ok(())
}
