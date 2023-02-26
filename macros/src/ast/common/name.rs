use crate::ast::{Attr, VecAttr};
use std::collections::BTreeSet;

pub struct Name {
    pub serialize: String,
    pub serialize_renamed: bool,
    pub deserialize: String,
    pub deserialize_renamed: bool,
    pub deserialize_aliases: Vec<String>,
}

impl Name {
    pub fn from_attrs(
        source_name: String,
        ser_name: Attr<String>,
        de_name: Attr<String>,
        de_aliases: Option<VecAttr<String>>,
    ) -> Name {
        let deserialize_aliases = match de_aliases {
            Some(de_aliases) => {
                let mut alias_list = BTreeSet::new();
                for alias_name in de_aliases.get() {
                    alias_list.insert(alias_name);
                }
                alias_list.into_iter().collect()
            }
            None => Vec::new(),
        };

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();

        Name {
            serialize: ser_name.unwrap_or_else(|| source_name.clone()),
            serialize_renamed: ser_renamed,
            deserialize: de_name.unwrap_or(source_name),
            deserialize_renamed: de_renamed,
            deserialize_aliases,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> String {
        self.serialize.clone()
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> String {
        self.deserialize.clone()
    }

    pub fn deserialize_aliases(&self) -> Vec<String> {
        let mut aliases = self.deserialize_aliases.clone();
        let main_name = self.deserialize_name();
        if !aliases.contains(&main_name) {
            aliases.push(main_name);
        }
        aliases
    }
}
