#![cfg(all(not(feature = "aio"), feature = "json"))]

use std::time::Duration;

use redis::Commands;
use redis_om::JsonModel;
use serde::{Deserialize, Serialize};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn client() -> Result<redis::Client> {
    Ok(redis::Client::open("redis://127.0.0.1/")?)
}

#[test]
fn basic_with_no_options() -> Result {
    #[derive(JsonModel, Serialize, Deserialize)]
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

    let mut conn = client()?.get_connection()?;

    account.save(&mut conn)?;

    let db_account = Account::get(&account.id, &mut conn)?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.last_name, db_account.last_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.id, &mut conn)?;

    Ok(())
}

#[test]
fn basic_with_custom_prefix_and_pk() -> Result {
    #[derive(JsonModel, Serialize, Deserialize)]
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

    let mut conn = client()?.get_connection()?;

    account.save(&mut conn)?;

    let count = conn
        .scan_match::<_, String>(format!("Account:{}", account.pk))?
        .count();

    assert_ne!(count, 0);

    let db_account = Account::get(&account.pk, &mut conn)?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.pk, &mut conn)?;

    Ok(())
}

#[test]
fn all_primary_keys() -> Result {
    #[derive(JsonModel, Serialize, Deserialize, Debug)]
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

    let mut conn = client()?.get_connection()?;

    for account in accounts.iter_mut() {
        account.save(&mut conn)?;
    }

    let pks = Account::all_pks(&mut conn)?.collect::<Vec<String>>();

    let count = pks.len();

    assert_eq!(count, 4);

    let db_accounts = pks
        .into_iter()
        .map(|id| Account::get(id, &mut conn))
        .collect::<Vec<_>>();

    for account in accounts.iter() {
        Account::delete(&account.id, &mut conn)?;
    }

    if !db_accounts.iter().all(|v| v.is_ok()) {
        panic!(
            "{:#?}",
            db_accounts
                .into_iter()
                .map(|v| v.unwrap_err())
                .collect::<Vec<_>>()
        );
    }

    Ok(())
}

#[test]
fn expiring_keys() -> Result {
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

    let mut conn = client()?.get_connection()?;

    customer.save(&mut conn)?;
    customer.expire(1, &mut conn)?;
    std::thread::sleep(Duration::from_secs(1));

    let count = Cusotmer::all_pks(&mut conn)?.count();

    assert_eq!(count, 0);

    Ok(())
}

#[test]
fn test_redis_schema() -> Result {
    #[derive(JsonModel, Serialize, Deserialize, Debug)]
    #[allow(dead_code)]
    struct Member {
        #[redis(index)]
        id: String,
        #[redis(index)]
        first_name: String,
        #[redis(index)]
        last_name: String,
        #[redis(index)]
        email: String,
        #[redis(index)]
        age: u32,
        #[redis(index, full_text_search, default = "String::default")]
        bio: Option<String>,
        join_date: String,
    }

    assert_eq!(
        Member::redissearch_schema(),
        &format!(
            "ON JSON PREFIX 1 Member SCHEMA \
               $.id AS id TAG SEPARATOR | \
               $.first_name AS first_name TAG SEPARATOR | \
               $.last_name AS last_name TAG SEPARATOR | \
               $.email AS email TAG SEPARATOR | \
               $.age AS age NUMERIC \
               $.bio AS bio TAG SEPARATOR | \
               $.bio AS bio_fts TEXT"
        )
    );

    Ok(())
}
