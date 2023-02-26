# redis-om-rust

The Unofficial Redis Object mapping that makes it easy to model Redis data in Rust. _inspired by [redis-om-python](https://github.com/redis/redis-om-python)_

## State

Alpha

## 💡 Why Redis OM?

Redis OM provides high-level abstractions that make it easy to model and query data in Redis with Rust.

## 📇 Modeling Your Data

Redis OM contains powerful declarative models that give you data validation, serialization, and persistence to Redis.

Check out this example of modeling customer data with Redis OM. First, we create a `Customer` model:

```rust
use redis_om::HashModel;

#[derive(HashModel, Debug, PartialEq, Eq)]
struct Customer {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub age: u32,
    pub bio: Option<String>
}

// Now that we have a `Customer` model, let's use it to save customer data to Redis.

let client = redis::Client::open("redis://127.0.0.1/").unwrap();

// First, we create a new `Customer` object:
let mut jane = Customer {
    id: "".into(), // will be auto generated when it's empty
    first_name: "Jane".into(),
    last_name: "Doe".into(),
    email: "jane.doe@example.com".into(),
    age: 40,
    bio: Some("Open Source Rust developer".into())
};

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

