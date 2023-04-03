# redis-om
[![MIT licensed][mit-badge]][mit-url]
[![Build status][gh-actions-badge]][gh-actions-url]
[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/redis-om.svg
[crates-url]: https://crates.io/crates/redis-om
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[gh-actions-badge]: https://github.com/kkharji/redis-om/workflows/Continuous%20integration/badge.svg
[gh-actions-url]: https://github.com/kkharji/redis-om/actions?query=workflow%3A%22Continuous+integration%22

A Rust/Redis ORM-style library that simplify the development process and reduce the amount of boilerplate code needed to build programs that leverage [redis] powerful capabilities and use cases.

**Status**: *WIP, fully testsed, possible breaking changes, stay tuned*

**Features**

- ORM-style API to define/manipulate [redis data structures] (e.g. hashes, json, streams) using [derive macros].
- Automatic serialization/desalinization between Redis data and rust objects.
- Interoperability with [serde](https://serde.rs/), e.g. using `rename`, `rename_all` or `serde`.
- Nested [hash datatype](#hash) support (e.g. `list.1` or nested models `account.balance` as keys).

**Usage**

- [Getting Started](#getting-started)
- [Using Redis's Hash Datatype](#hash)
- [Using Redis's Json Datatype](#json)
- [Using Redis's Stream Datatype](#stream)

**Roadmap**

- <kbd>0.1.0</kbd>
    - [x] Enable users to define and derive Hash Model with most common methods
    - [x] Enable users to define and derive JSON Model with most common methods
    - [x] Enable users to define and derive streams with managers to publish-to/read-from them.
    - [x] Support users to choose between asynchronous and synchronous runtime.
- <kbd>0.2.0</kbd>
    - [ ] Enable Multi-Stream Manager Support to enable users to combine multiple `RedisModels`.
    - [ ] Support Serializing/deserializing `HashModel` complex fields using serde.
    - [ ] Support `RedisSearch` and provide query-building API.
    - [ ]  .....
- <kbd>0.3.0</kbd>
    - [ ] Support validation of struct fields and enum values (most likely using [validator library]).
    - [ ] .....


## Getting Started

```toml
redis-om = { version = "*" }
# TLS support with async-std
redis-om = { version = "*", features = ["tls"] }
# async support with tokio
redis-om = { version = "*", features = ["tokio-comp"] }
# async support with async-std
redis-om = { version = "*", features = ["async-std-comp"] }
# TLS and async support with tokio
redis-om = { version = "*", features = ["tokio-native-tls-comp"] }
# TLS support with async-std
redis-om = { version = "*", features = ["async-std-tls-comp"] }
```

## Hash

```rust ignore
use redis_om::HashModel;

#[derive(HashModel, Debug, PartialEq, Eq)]
struct Customer {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
    bio: Option<String>,
    interests: Vec<String>
}

// Now that we have a `Customer` model, let's use it to save customer data to Redis.

// First, we create a new `Customer` object:
let mut jane = Customer {
    id: "".into(), // will be auto generated when it's empty
    first_name: "Jane".into(),
    last_name: "Doe".into(),
    email: "jane.doe@example.com".into(),
    bio: Some("Open Source Rust developer".into()),
    interests: vec!["Books".to_string()],
};

// Get client
let client = redis_om::Client::open("redis://127.0.0.1/").unwrap();
// Get connection
let mut conn = client.get_connection().unwrap();

// We can save the model to Redis by calling `save()`:
jane.save(&mut conn).unwrap();

// Expire the model after 1 min (60 seconds)
jane.expire(60, &mut conn).unwrap();

// Retrieve this customer with its primary key
let jane_db = Customer::get(&jane.id, &mut conn).unwrap();

// Delete customer
Customer::delete(&jane.id, &mut conn).unwrap();

assert_eq!(jane_db, jane);
```

## Json

redis-om support json data type through `redis_om::JsonModel`. It requires that the type
derives `serde::Deserialize` as well as `serde::Serialize`.

```rust ignore
use redis_om::JsonModel;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct AccountDetails {
    balance: String,
}

#[derive(JsonModel, Deserialize, Serialize, Debug, PartialEq, Eq)]
struct Account {
    id: String,
    first_name: String,
    last_name: String,
    details: AccountDetails,
}

// Now that we have a `Account` model, let's use it to save account data to Redis.

// First, we create a new `Account` object:
let mut john = Account {
    id: "".into(), // will be auto generated when it's empty
    first_name: "John".into(),
    last_name: "Doe".into(),
    details: AccountDetails {
        balance: "1.5m".into(),
    }
};

// Get client
let client = redis_om::Client::open("redis://127.0.0.1/").unwrap();
// Get connection
let mut conn = client.get_connection().unwrap();

// We can save the model to Redis by calling `save()`:
john.save(&mut conn).unwrap();

// Expire the model after 1 min (60 seconds)
john.expire(60, &mut conn).unwrap();

// Retrieve this account with its primary key
let john_db = Account::get(&john.id, &mut conn).unwrap();

// Delete customer
Account::delete(&john.id, &mut conn).unwrap();

assert_eq!(john_db, john);
```
## Stream

redis-om support json data type through `redis_om::StreamModel`. It requires that any nested type to derives `redis_om::RedisTransportValue`.

```rust ignore
use redis_om::{RedisTransportValue, StreamModel};

/// An enum of room service kind
#[derive(RedisTransportValue)]
pub enum RoomServiceJob {
    Clean,
    ExtraTowels,
    ExtraPillows,
    FoodOrder,
}

/// An enum of room service kind
#[derive(StreamModel)]
#[redis(key = "room")] // rename stream key in redis
pub struct RoomServiceEvent {
    status: String,
    room: usize,
    job: RoomServiceJob,
}

// Get client
let client = redis_om::Client::open("redis://127.0.0.1/").unwrap();
// Get connection
let mut conn = client.get_connection().unwrap();

// Create a new instance of Room service Event Manager with consumer group.
// Note: consumer name is auto generated,
// use RoomServiceEventManager::new_with_consumer_name, // for a custom name
let manager = RoomServiceEventManager::new("Staff");

// Ensure the consumer group
manager.ensure_group_stream(&mut conn).unwrap();

// Create new event
let event = RoomServiceEvent {
    status: "pending".into(),
    room: 3,
    job: RoomServiceJob::Clean,
};

// Publish the event to the RoomServiceEvent redis stream
RoomServiceEventManager::publish(&event, &mut conn).unwrap();

// Read with optional read_count: Option<usize>, block_interval: Option<usize>
let read = manager.read(None, None, &mut conn).unwrap();

// Get first incoming event
let incoming_event = read.first().unwrap();
// Get first incoming event data
let incoming_event_data = incoming_event.data::<RoomServiceEvent>().unwrap();
// Acknowledge that you received the event, so other in the consumers don't get it
RoomServiceEventManager::ack(manager.group_name(), &[&incoming_event.id], &mut conn).unwrap();

assert_eq!(incoming_event_data.room, event.room);
```

[derive macros]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros
[redis data structures]: https://redis.com/redis-enterprise/data-structures/
[redis]: https://redis.com
[validator library]: https://crates.io/crates/validator
