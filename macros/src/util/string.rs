use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};

/// Transforms a value into the desired case style.
///
/// This function panics if an invalid `casing` value is provided.
pub fn transform_casing(value: &str, casing: Option<&str>) -> String {
    let Some(rule) = casing else { return value.into() };
    match rule {
        "lowercase" => value.to_lowercase(),
        "UPPERCASE" => value.to_uppercase(),
        "PascalCase" => value.to_pascal_case(),
        "camelCase" => value.to_lower_camel_case(),
        "snake_case" => value.to_snake_case(),
        "kebab-case" => value.to_kebab_case(),
        _ => panic!("Invalid rename_all value: supported: kebab-case, snake_case, camelCase, PascalCase, lowercase, UPPERCASE"),
    }
}
