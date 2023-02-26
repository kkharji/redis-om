use redis_om::{RedisModel, RedisTransportValue};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[test]
fn basic_with_no_options() -> Result {
    #[derive(RedisTransportValue, RedisModel)]
    struct Account {
        id: String,
        first_name: String,
    }

    let mut account = Account {
        id: "".into(),
        first_name: "Joe".into(),
    };

    assert_eq!(Account::_prefix_key(), "Account");
    assert_eq!(account.id, "");
    account._set_pk("1234".to_string());
    assert_eq!(account.id, "1234");

    Ok(())
}

#[test]
fn basic_with_custom_prefix_and_pk() -> Result {
    #[derive(RedisTransportValue, RedisModel)]
    #[redis(prefix_key = "info_details")]
    struct Details {
        #[redis(primary_key)]
        pk: String,
        city: String,
    }

    let mut details = Details {
        pk: "".into(),
        city: "Joe".into(),
    };

    assert_eq!(Details::_prefix_key(), "info_details");
    assert_eq!(details.pk, "");
    details._set_pk("1234".to_string());
    assert_eq!(details.pk, "1234");

    Ok(())
}
