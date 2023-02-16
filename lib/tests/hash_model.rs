use redis_om::redis::Value;
use redis_om::HashModel;
use redis_om::RedisTransportValue;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[test]
fn basic_with_no_options() -> Result {
    #[derive(HashModel)]
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

    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut conn = client.get_connection()?;

    account.save(&mut conn)?;

    let db_account = Account::get(&account.id, &mut conn)?;

    assert_eq!(account.first_name, db_account.first_name);
    assert_eq!(account.last_name, db_account.last_name);
    assert_eq!(account.interests, db_account.interests);

    Account::delete(account.id, &mut conn)?;

    Ok(())
}
