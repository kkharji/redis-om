use syn::{punctuated::Punctuated, token::Comma, Field, Fields};

pub trait FieldsExt {
    fn named(&self) -> &Punctuated<Field, Comma>;
}

impl FieldsExt for Fields {
    fn named(&self) -> &Punctuated<Field, Comma> {
        let Fields::Named(fields) = self else { panic!("Expceted named field") };
        &fields.named
    }
}
