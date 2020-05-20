use std::convert::TryFrom;

use proc_macro2::TokenStream;
use syn::{Attribute, Error, Field, Lit, LitStr, Meta, Type};

#[derive(Debug)]
pub enum Endian {
    Little,
    Middle,
    MiddleInverse,
}

impl TryFrom<&LitStr> for Endian {
    type Error = TokenStream;

    fn try_from(value: &LitStr) -> Result<Self, Self::Error> {
        match value.value().as_str() {
            "little" => Ok(Self::Little),
            "middle" => Ok(Self::Middle),
            "middle-inverse" => Ok(Self::MiddleInverse),
            _ => Err(Error::new_spanned(
                value,
                "expected one of \"little\", \"middle\" or \"middle-inverse\"",
            )
            .to_compile_error()),
        }
    }
}

pub fn parse_endianness(field: &Field, attr: &Attribute) -> Result<Endian, TokenStream> {
    let ty = match &field.ty {
        Type::Path(path) => &path.path.segments,
        _ => panic!("unexpected token"),
    };

    let ty_name = &ty.first().unwrap().ident;

    let name_value = match attr.parse_meta() {
        Ok(Meta::NameValue(name_value)) => name_value,
        Ok(meta) => {
            return Err(syn::Error::new_spanned(
                meta,
                "usage: #[endian = \"<little|middle|middle-inverse>\"]",
            )
            .to_compile_error());
        }
        Err(e) => {
            return Err(
                syn::Error::new(e.span(), "failed to parse endian attribute").to_compile_error(),
            );
        }
    };

    let (literal, endianness) = match name_value.lit {
        Lit::Str(ref literal) => (literal, Endian::try_from(literal)?),
        lit => {
            return Err(syn::Error::new_spanned(
                lit,
                "expected one of \"little\", \"middle\", \"middle-inverse\"",
            )
            .to_compile_error());
        }
    };

    match endianness {
        Endian::Middle | Endian::MiddleInverse => {
            if ty_name != "u32" {
                let message = format!("{} endian can only be applied to u32", literal.value());
                return Err(syn::Error::new_spanned(literal, message).to_compile_error());
            }
        }
        _ => {}
    }

    Ok(endianness)
}
