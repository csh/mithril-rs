use std::convert::TryFrom;

use proc_macro2::{TokenStream, Span, Punct, Spacing};
use syn::{Attribute, Error, Lit, LitStr, Meta};
use quote::{ToTokens, TokenStreamExt};

#[derive(Debug)]
pub enum Transform {
    Add,
    Negate,
    Subtract,
}

impl ToTokens for Transform {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(syn::Ident::new("Transform", Span::call_site()));
        let string = match self {
            Transform::Add => "Add",
            Transform::Negate => "Subtract",
            Transform::Subtract => "Negate",
        };
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(syn::Ident::new(string, Span::call_site()));
    }
}

impl TryFrom<LitStr> for Transform {
    type Error = TokenStream;

    fn try_from(value: LitStr) -> Result<Self, Self::Error> {
        match value.value().as_str() {
            "add" => Ok(Self::Add),
            "negate" => Ok(Self::Negate),
            "subtract" => Ok(Self::Subtract),
            _ => Err(Error::new_spanned(
                value,
                "expected one of \"add\", \"negate\" or \"subtract\"",
            )
            .to_compile_error()),
        }
    }
}

pub fn parse_transform(attr: &Attribute) -> Result<Transform, TokenStream> {
    let name_value = match attr.parse_meta() {
        Ok(Meta::NameValue(name_value)) => name_value,
        Ok(meta) => {
            return Err(syn::Error::new_spanned(
                meta,
                "usage: #[transform = \"<add|negate|subtract>\"]",
            )
            .to_compile_error());
        }
        Err(e) => {
            return Err(
                syn::Error::new(e.span(), "failed to parse transform attribute").to_compile_error(),
            );
        }
    };

    let transform = match name_value.lit {
        Lit::Str(literal) => Transform::try_from(literal),
        lit => {
            return Err(syn::Error::new_spanned(
                lit,
                "expected one of \"add\", \"subtract\", or \"negate\"",
            )
            .to_compile_error());
        }
    }?;

    Ok(transform)
}
