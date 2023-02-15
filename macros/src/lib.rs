#![deny(missing_docs, unstable_features)]

//! Derive proc macros for redis-om crate

mod util;
mod value;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data::*, DeriveInput};
use value::DeriveRedisValue;

/// Derive procedural macro that automatically generate implementation for
///
/// [`RedisTransportValue`](redis_om::RedisTransportValue), which requires implementation of the following:
///
/// - [`ToRedisArgs`](redis::ToRedisArgs),
/// - [`FromRedisValue`](redis::FromRedisValue)
///
/// # Example
///
/// ```
/// use redis_om::RedisTransportValue;
///
/// #[derive(RedisTransportValue)]
/// struct Test {
///     field_one: String,
///     field_two: i32,
///     field_three: Vec<u8>,
/// }
///
/// let test = Test {
///     field_one: "Hello, World!".to_owned(),
///     field_two: 42,
///     field_three: vec![1, 2, 3, 4, 5],
/// };
///
/// use redis_om::redis::{Value};
///
/// let serialized: Vec<Value> = test.to_redis_args().into_iter().map(|v| Value::Data(v)).collect();
/// let deserialized: Test = Test::from_redis_value(&Value::Bulk(serialized)).unwrap();
///
/// assert_eq!(test.field_one, deserialized.field_one);
/// assert_eq!(test.field_two, deserialized.field_two);
/// assert_eq!(test.field_three, deserialized.field_three);
/// ```
///
/// # Attributes
///
/// ## `rename_all = "some_case"`
///
/// This attribute sets the default casing to be used for field names when generating Redis command arguments.
/// Possible values are:
///
/// - `"snake_case"`: field names will be converted to `snake_case`
/// - `"kebab-case"`: field names will be converted to `kebab-case`
/// - `"camelCase"`: field names will be converted to `camelCase`
/// - `"PascalCase"`: field names will be converted to `PascalCase`
///
/// This attribute can be overridden for individual fields using the `rename` attribute.
///
/// ## `rename = "some_field"`
///
/// This attribute sets the Redis key name to be used for a field when generating Redis command arguments.
/// This attribute takes precedence over the `rename_all` attribute.
///
/// # Restrictions
///
/// - Enums with fields are not supported
/// - Generics are not supported
/// - Public fields are required
/// - Fields with types that do not implement `ToRedisArgs` and `FromRedisValue` are not supported
///
#[proc_macro_derive(RedisTransportValue, attributes(redis))]
pub fn redis_transport_value(attr: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(attr as DeriveInput);
    let ident = ast.ident;
    let attrs = util::parse::attributes(&ast.attrs);

    match ast.data {
        Struct(s) => s.derive_redis_value(&ident, &attrs).into(),
        Enum(e) => e.derive_redis_value(&ident, &attrs).into(),
        Union(_) => panic!("Unions is not supported. Please open an issue in https://github.com/kkharji/redis-om-rust"),
    }
}
