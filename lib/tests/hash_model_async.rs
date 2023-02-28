#![cfg(all(feature = "tokio-comp", feature = "json"))]

use std::time::Duration;

use redis::AsyncCommands;
use redis_om::redis::Value;
use redis_om::redis::{FromRedisValue, ToRedisArgs};
use redis_om::{HashModel, RedisResult};
use tokio::test;

use futures::StreamExt;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

async fn conn() -> RedisResult<redis::aio::Connection> {
    redis::Client::open("redis://127.0.0.1/")?
        .get_async_connection()
        .await
}

#[test]
async fn basic_with_no_options() -> Result {
    #[derive(HashModel)]
    #[redis(prefix_key = "users")]
    struct Account {
        id: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut account = Account {
        id: "".into(),
        first_name: "Joe".into(),
        last_name: "Doe".into(),
        interests: vec!["Gaming".into(), "SandCasting".into(), "Writing".into()],
    };

    let serialized = account
        .to_redis_args()
        .into_iter()
        .map(|v| Value::Data(v))
        .collect::<Vec<_>>();
    let deserialized = Account::from_redis_value(&Value::Bulk(serialized))?;

    // Ensure that values are identical
    assert_eq!(account.first_name, deserialized.first_name);
    assert_eq!(account.last_name, deserialized.last_name);
    assert_eq!(account.interests, deserialized.interests);

    let mut conn = conn().await?;

    account.save(&mut conn).await?;

    let db_account = Account::get(&account.id, &mut conn).await?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.last_name, db_account.last_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.id, &mut conn).await?;

    Ok(())
}

#[test]
async fn basic_with_custom_prefix_and_pk() -> Result {
    #[derive(HashModel)]
    struct Account {
        #[redis(primary_key)]
        pk: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut account = Account {
        pk: "".into(),
        first_name: "Joe".into(),
        last_name: "Doe".into(),
        interests: vec!["Gaming".into(), "SandCasting".into(), "Writing".into()],
    };

    let mut conn = conn().await?;

    account.save(&mut conn).await?;

    let count = conn
        .scan_match::<_, String>(format!("Account:{}", account.pk))
        .await?
        .count()
        .await;

    assert_ne!(count, 0);

    let db_account = Account::get(&account.pk, &mut conn).await?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.pk, &mut conn).await?;

    Ok(())
}

#[test]
async fn all_primary_keys() -> Result {
    #[derive(HashModel, Debug)]
    #[redis(prefix_key = "accounts")]
    struct Account {
        id: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut accounts = [
        ["Joe", "Doe"],
        ["Jane", "Doe"],
        ["John", "Smith"],
        ["Tim", "B"],
    ]
    .map(|[first, last]| Account {
        id: "".into(),
        first_name: first.into(),
        last_name: last.into(),
        interests: vec!["Gaming".into(), "SandCasting".into(), "Writing".into()],
    })
    .into_iter()
    .collect::<Vec<_>>();

    let mut conn = conn().await?;

    for account in accounts.iter_mut() {
        account.save(&mut conn).await?;
    }

    let pks = Account::all_pks(&mut conn)
        .await?
        .collect::<Vec<String>>()
        .await;

    let count = pks.len();

    assert_eq!(count, 4);

    for account in accounts.iter() {
        Account::delete(&account.id, &mut conn).await?;
    }

    Ok(())
}

#[test]
async fn expiring_keys() -> Result {
    #[derive(HashModel)]
    #[redis(prefix_key = "customers")]
    struct Cusotmer {
        #[redis(primary_key)]
        pk: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut customer = Cusotmer {
        pk: "".into(),
        first_name: "Joe".into(),
        last_name: "Doe".into(),
        interests: vec!["Gaming".into(), "SandCasting".into(), "Writing".into()],
    };

    let mut conn = conn().await?;

    customer.save(&mut conn).await?;
    customer.expire(1, &mut conn).await?;
    std::thread::sleep(Duration::from_secs(1));

    let count = Cusotmer::all_pks(&mut conn).await?.count().await;

    assert_eq!(count, 0);

    Ok(())
}
