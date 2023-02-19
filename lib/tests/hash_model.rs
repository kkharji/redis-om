use redis::Commands;
use redis_om::redis::Value;
use redis_om::redis::{FromRedisValue, ToRedisArgs};
use redis_om::HashModel;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn client() -> Result<redis::Client> {
    Ok(redis::Client::open("redis://127.0.0.1/")?)
}

#[test]
fn basic_with_no_options() -> Result {
    #[derive(HashModel)]
    #[redis(prefix = "users")]
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
    #[derive(HashModel)]
    #[redis(key_prefix = "user", pk_field = "pk")]
    struct Account {
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
        .scan_match::<_, String>(format!("user:{}", account.pk))?
        .count();

    assert_ne!(count, 0);

    let db_account = Account::get(&account.pk, &mut conn)?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.pk, &mut conn)?;

    Ok(())
}

#[test]
fn basic_with_getters_setters() -> Result {
    #[derive(HashModel, Default)]
    #[redis(key_prefix = "user", pk_field = "pk")]
    struct Account {
        pk: String,
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let mut account = Account::default();

    account
        .set_first_name("Joe")
        .set_last_name("Doe")
        .set_interests(vec![
            "Gaming".into(),
            "SandCasting".into(),
            "Writing".into(),
        ]);

    let mut conn = client()?.get_connection()?;

    account.save(&mut conn)?;

    let count = conn
        .scan_match::<_, String>(format!("user:{}", account.pk()))?
        .count();

    assert_ne!(count, 0);

    let db_account = Account::get(&account.pk(), &mut conn)?;

    assert_eq!(account.first_name(), db_account.first_name());
    assert_eq!(account.interests(), db_account.interests());

    Account::delete(account.pk(), &mut conn)?;

    Ok(())
}
