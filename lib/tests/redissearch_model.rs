use redis_om::RedisSearchModel;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[test]
fn test_schema() -> Result {
    #[derive(RedisSearchModel, Debug)]
    #[allow(dead_code)]
    #[redis(model_type = "HashModel")]
    struct Address {
        #[redis(index)]
        id: String,
        #[redis(index)]
        a: String,
        #[redis(index, full_text_search)]
        b: String,
        #[redis(index, sortable)]
        c: u32,
        #[redis(index)]
        d: f32,
    }

    assert_eq!(
        Address::REDIS_SEARCH_SCHEMA,
        format!(
            "ON HASH PREFIX 1 Address SCHEMA \
                id TAG SEPARATOR | \
                a TAG SEPARATOR | \
                b TAG SEPARATOR | \
                b AS b_fts TEXT \
                c NUMERIC SORTABLE \
                d NUMERIC"
        )
    );

    Ok(())
}
