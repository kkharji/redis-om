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

#[test]
fn all_primary_keys() -> Result {
    #[derive(HashModel, Debug)]
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
        Account::delete(account.id(), &mut conn)?;
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
fn test_schema() {
    #[derive(HashModel, Debug)]
    struct Address {
        id: String,
        #[redis(index)]
        a: String,
        #[redis(index, full_text_search)]
        b: String,
        #[redis(index, sortable)]
        integer: u32,
        #[redis(index)]
        float: f32,
    }

    let key_prefix = Address::redis_prefix();
    let schema = Address::redissearch_schema().to_string();

    assert_eq!(
        schema,
        format!(
            "ON HASH PREFIX 1 {key_prefix} SCHEMA id TAG SEPARATOR | \
                a TAG SEPARATOR | \
                b TAG SEPARATOR | \
                b AS b_fts TEXT \
                integer NUMERIC SORTABLE \
                float NUMERIC"
        )
    );
}
