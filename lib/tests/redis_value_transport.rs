use redis_om::redis::{from_redis_value, Value};
use redis_om::redis::{FromRedisValue, ToRedisArgs};
use redis_om::RedisTransportValue;
use std::collections::HashMap;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[test]
fn struct_with_no_options() -> Result {
    #[derive(RedisTransportValue)]
    struct Account {
        first_name: String,
        last_name: String,
        interests: Vec<String>,
    }

    let account = Account {
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

    Ok(())
}

#[test]
fn struct_with_rename_all_option() -> Result {
    #[derive(RedisTransportValue)]
    #[redis(rename_all = "camelCase")]
    struct Account {
        first_name: String,
        last_name: String,
    }

    let account = Account {
        first_name: "Joe".into(),
        last_name: "Doe".into(),
    };

    let serialized = Value::Bulk(
        account
            .to_redis_args()
            .into_iter()
            .map(|v| Value::Data(v))
            .collect::<Vec<_>>(),
    );

    // Ensure that values are identical
    let deserialized = Account::from_redis_value(&serialized)?;
    assert_eq!(account.first_name, deserialized.first_name);
    assert_eq!(account.last_name, deserialized.last_name);

    // Ensure that fields are renamed
    let account_map: HashMap<String, String> = from_redis_value(&serialized)?;
    assert_eq!(account_map["firstName"], account.first_name);
    assert_eq!(account_map["lastName"], account.last_name);

    Ok(())
}

#[test]
fn enum_with_no_options() -> Result {
    #[derive(Debug, RedisTransportValue, PartialEq)]
    enum State {
        On,
        Off,
    }

    let state = State::On;

    let serialized = state.to_redis_args().first().unwrap().to_vec();
    let deserialized = State::from_redis_value(&Value::Data(serialized))?;

    // Ensure that values are identical
    assert_eq!(deserialized, state);

    Ok(())
}

#[test]
fn enum_with_rename_all_option() -> Result {
    #[derive(Debug, RedisTransportValue, PartialEq)]
    #[redis(rename_all = "kebab-case")]
    enum State {
        OnDevice,
        OffDevice,
    }

    let state = State::OffDevice;

    let serialized = state.to_redis_args().first().unwrap().to_vec();
    let data = Value::Data(serialized.clone());
    let stringified = String::from_utf8(serialized)?;
    let deserialized = State::from_redis_value(&data)?;

    // Ensure that value is lowercase
    assert_eq!(stringified.as_str(), "off-device");
    // Ensure that values are identical
    assert_eq!(deserialized, state);

    Ok(())
}

#[test]
fn enum_struct_compo() -> Result {
    #[derive(Debug, RedisTransportValue, PartialEq)]
    enum AccountKind {
        Admin,
        Shopper,
    }

    #[derive(RedisTransportValue)]
    #[redis(rename_all = "camelCase")]
    struct Account {
        first_name: String,
        last_name: String,
        #[redis(rename = "accountKind")]
        kind: AccountKind,
    }

    let account = Account {
        first_name: "Joe".into(),
        last_name: "Doe".into(),
        kind: AccountKind::Shopper,
    };

    let serialized = Value::Bulk(
        account
            .to_redis_args()
            .into_iter()
            .map(|v| Value::Data(v))
            .collect::<Vec<_>>(),
    );

    // Ensure that values are identical
    let deserialized = Account::from_redis_value(&serialized)?;
    assert_eq!(account.first_name, deserialized.first_name);
    assert_eq!(account.last_name, deserialized.last_name);
    assert_eq!(account.kind, deserialized.kind);

    // Ensure that fields are renamed
    let account_map: HashMap<String, String> = from_redis_value(&serialized)?;
    assert_eq!(account_map["firstName"], account.first_name);
    assert_eq!(account_map["lastName"], account.last_name);
    assert_eq!(account_map["accountKind"], "Shopper");

    Ok(())
}
