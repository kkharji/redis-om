use std::collections::HashMap;
use syn::{Attribute, Meta, NestedMeta};

pub type AttributeMap = HashMap<String, String>;

/// Parses the Redis attributes in the given list of attributes, and returns a
/// mapping of attribute names to their string values.
/// TODO: PANIC if key is not recognized
pub fn attributes(attributes: &[Attribute]) -> AttributeMap {
    let mut attr_map = HashMap::new();
    for attribute in attributes {
        if !attribute.path.is_ident("redis") {
            continue;
        }

        if let Ok(Meta::List(meta)) = attribute.parse_meta() {
            if meta.path.is_ident("redis") {
                for nested_meta in meta.nested {
                    if let NestedMeta::Meta(Meta::Path(ref path)) = nested_meta {
                        attr_map.insert(
                            path.get_ident().expect("Attribute name").to_string(),
                            "true".to_string(),
                        );
                    }

                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                        let attr_name = name_value
                            .path
                            .get_ident()
                            .expect("Attribute name expected")
                            .to_string();
                        let attr_value = match &name_value.lit {
                            syn::Lit::Str(lit_str) => lit_str.value(),
                            syn::Lit::Bool(lit_bool) => lit_bool.value().to_string(),
                            _ => panic!(
                                "Attribute value must be a string literal or boolean literal"
                            ),
                        };

                        if attr_name == "index" {
                            if let Ok(val) = attr_value.parse() {
                                attr_map.insert("index".to_string(), val);
                                continue;
                            } else {
                                panic!("Index attribute value must be a boolean literal");
                            }
                        }
                        attr_map.insert(attr_name, attr_value);
                    }
                }
            }
        }
    }
    attr_map
}
