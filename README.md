# redis-om-rust

The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

## State

Alpha

## 💡 Why Redis OM?

Redis OM provides high-level abstractions that make it easy to model and query data in Redis with Rust.

## 📇 Modeling Your Data

Redis OM contains powerful declarative models that give you data serialization and persistence to Redis.

Check out usage section for example on using redis-om-rust in rust applications.

## Features

- [serde](https://serde.rs/) interop annotations such as `rename`, `rename_all`, alias and many more.
- Use struct methods todo all kind of crud and redis specific operations.
- Serialize [hash](#hash) model list-like and dict-like structs as prefix keys without needing JSON (i.e. list.1, account.balance).
- Support for [json](#json) datatype
- Support for [stream](#json) datatype

## Usage

- [hash datatype macro usage](#Hash)
- [Json datatype macro usage](#json)
- [Stream datatype macro usage](#Hash)

### Hash

```rust
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
let client = redis::Client::open("redis://127.0.0.1/").unwrap();
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

### Json

redis-om support json data type through `redis_om::JsonModel`. It requires that the type
derives `serde::Deserialize` as well as `serde::Serialize`.

```rust
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
let client = redis::Client::open("redis://127.0.0.1/").unwrap();
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
### Stream

redis-om support json data type through `redis_om::StreamModel`. It requires that any nested type to derives `redis_om::RedisTransportValue`.

```rust
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
let client = redis::Client::open("redis://127.0.0.1/").unwrap();
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
// Get first incoming event data
let incoming_event = read.first().unwrap().data::<RoomServiceEvent>().unwrap();

assert_eq!(incoming_event.room, event.room);
```

## Roadmap

### 0.1.0

- [x] Hash Models
- [x] Json Model
- [x] Stream Model
- [x] Async support

### 0.2.0
- [ ] Multi Stream Manager
- [ ] stream model enum support
- [ ] serializing/deserializing hash model fields using serde for hash models
- [ ] Correctly support RedisSearch Integration with embedded types
- [ ] Internal managed connections, i.e. no requirement to pass conn around.
- [ ] Values Validation Support
