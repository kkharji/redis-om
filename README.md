# redis-om-rust

The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

## State

Alpha

## Async Support

To enable asynchronous clients a feature for the underlying feature need to be activated.

```toml
# if you use tokio
redis-om = { version = "*", features = ["tokio-comp"] }

# if you use async-std
redis-om = { version = "*", features = ["async-std-comp"] }
```

## TLS Support
```toml
redis-om = { version = "*", features = ["tls"] }

# if you use tokio
redis-om = { version = "*", features = ["tokio-native-tls-comp"] }

# if you use async-std
redis-om = { version = "*", features = ["async-std-tls-comp"] }
```

## Usage

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

## Features

- serde interop annotations such as rename, rename_all, alias and many more.
- Use struct static function todo all the required crud operations.
- Serialize hash model list-like and dict-like structs as prefix keys without needing JSON
  (i.e. list.1, account.balance).


## Roadmap

### 0.1.0

- [x] Hash Models
- [x] Json Model
- [x] Async support
- [ ] Stream Model
- [ ] serializing/deserializing hash model fields using serde for hash models

### 0.2.0
- [ ] Correctly support RedisSearch Integration with embedded types
- [ ] Internal managed connections, i.e. no requirement to pass conn around.
- [ ] Values Validation Support

### 0.3.0
- [ ] List Model
