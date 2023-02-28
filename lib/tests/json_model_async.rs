#![cfg(all(feature = "tokio-comp", feature = "json"))]
use std::time::Duration;
use tokio::test;

use redis::AsyncCommands;
use redis_om::{JsonModel, RedisResult};
use serde::{Deserialize, Serialize};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

async fn conn() -> RedisResult<redis::aio::Connection> {
    redis::Client::open("redis://127.0.0.1/")?
        .get_async_connection()
        .await
}

#[test]
async fn basic_with_no_options() -> Result {
    #[derive(JsonModel, Serialize, Deserialize)]
    #[redis(prefix_key = "users")]
    struct Account {
        id: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut conn = conn().await?;
    let mut account = Account {
        id: "".into(),
        first_name: "Joe".into(),
        last_name: "Doe".into(),
        interests: vec!["Gaming".into(), "SandCasting".into(), "Writing".into()],
    };

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
    #[derive(JsonModel, Serialize, Deserialize)]
    struct Account {
        #[redis(primary_key)]
        pk: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut conn = conn().await?;
    let mut account = Account {
        pk: "".into(),
        first_name: "Joe".into(),
        last_name: "Doe".into(),
        interests: vec!["Gaming".into(), "SandCasting".into(), "Writing".into()],
    };

    account.save(&mut conn).await?;

    let mut count = 0;
    let mut iter = conn
        .scan_match::<_, String>(format!("Account:{}", account.pk))
        .await?;

    while let Some(_) = iter.next_item().await {
        count += 1;
    }

    assert_ne!(count, 0);

    let db_account = Account::get(&account.pk, &mut conn).await?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.pk, &mut conn).await?;

    Ok(())
}

#[test]
async fn all_primary_keys() -> Result {
    #[derive(JsonModel, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[redis(prefix_key = "accounts")]
    struct Account {
        id: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut conn = conn().await?;
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

    for account in accounts.iter_mut() {
        account.save(&mut conn).await?;
    }

    let mut pks = vec![];
    let mut iter = Account::all_pks(&mut conn).await?;

    while let Some(item) = iter.next_item().await {
        pks.push(item)
    }

    let count = pks.len();

    assert_eq!(count, 4);

    let mut db_accounts = vec![];

    for pk in pks {
        db_accounts.push(Account::get(pk, &mut conn).await?);
    }

    for account in accounts.iter() {
        Account::delete(&account.id, &mut conn).await?;
    }

    Ok(())
}

#[test]
async fn expiring_keys() -> Result {
    #[derive(JsonModel, Serialize, Deserialize)]
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
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut iter = Cusotmer::all_pks(&mut conn).await?;
    let mut pks = vec![];

    while let Some(item) = iter.next_item().await {
        pks.push(item)
    }

    assert_eq!(pks.len(), 0);

    Ok(())
}
