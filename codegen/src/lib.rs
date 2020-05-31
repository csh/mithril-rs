use proc_macro2::{TokenStream, Span};
use std::collections::HashMap;

use quote::quote;
use once_cell::sync::Lazy;
use syn::{Ident, DeriveInput};

use crate::endian::Endian;

mod endian;
mod transform;

static ALLOWED_BY_ATTR: Lazy<HashMap<&'static str, Vec<String>>> = Lazy::new(|| {
    let mut allowed = HashMap::new();

    allowed.insert("base37", vec![String::from("String")]);

    allowed.insert("endian", vec![String::from("u16"), String::from("u32")]);

    allowed.insert(
        "transform",
        vec![
            String::from("bool"),
            String::from("u8"),
            String::from("u16"),
        ],
    );

    allowed
});

#[derive(Debug, Default)]
struct FieldMetadata {
    transform: Option<transform::Transform>,
    endian: Option<endian::Endian>,
    base37: bool,
}

fn generate_fn(prefix: &str, field_type: &Ident, field_metadata: &FieldMetadata) -> Ident {
    let mut fn_name = format!("{}_{}", prefix, field_type);
    if field_metadata.transform.is_some() {
        fn_name.push('t');
    }
    if let Some(ref endian) = field_metadata.endian {
        match endian {
            Endian::Little => fn_name.push_str("_le"),
            Endian::Middle => fn_name.push_str("_me"),
            Endian::MiddleInverse => fn_name.push_str("_inv_me"),
        }
    }
    Ident::new(&fn_name, Span::call_site())
}

fn generate_quote_read(field_name: &Ident, field_type: &Ident, field_metadata: &FieldMetadata) -> TokenStream {
    if field_type == "bool" {
        if let Some(ref transform) = field_metadata.transform {
            quote! {
                self.#field_name = src.get_u8t(#transform) == 1;
            }
        } else {
            quote! {
                self.#field_name = src.get_u8() == 1;
            }
        }
    } else if field_type == "String" {
        if field_metadata.base37 {
            quote! {
                let base37_encoded = src.get_u64();
                self.#field_name = mithril_text::decode_base37(base37_encoded)?;
            }
        } else {
            quote! {
                self.#field_name = src.get_rs_string();
            }
        }
    } else {
        let fn_ident = generate_fn("get", field_type, &field_metadata);
        if let Some(ref transform) = field_metadata.transform {
            quote! {
                self.#field_name = src.#fn_ident(#transform);
            }
        } else {
            quote! {
                self.#field_name = src.#fn_ident();
            }
        }
    }
}

fn generate_quote_write(field_name: &Ident, field_type: &Ident, field_metadata: &FieldMetadata) -> TokenStream {
    if field_type == "bool" {
        if let Some(ref transform) = field_metadata.transform {
            quote! {
                dst.put_u8t(if self.#field_name == true { 1 } else { 0 }, #transform);
            }
        } else {
            quote! {
                dst.put_u8(if self.#field_name == true { 1 } else { 0 });
            }
        }
    } else if field_type == "String" {
        if field_metadata.base37 {
            quote! {
                let base37_encoded = mithril_text::encode_base37(self.#field_name.clone());
                dst.put_u64(base37_encoded);
            }
        } else {
            quote! {
                dst.put_rs_string(self.#field_name.clone());
            }
        }
    } else {
        let fn_ident = generate_fn("put", field_type, &field_metadata);
        if let Some(ref transform) = field_metadata.transform {
            quote! {
                dst.#fn_ident(self.#field_name, #transform);
            }
        } else {
            quote! {
                dst.#fn_ident(self.#field_name);
            }
        }
    }
}

#[proc_macro_derive(Packet, attributes(transform, base37, endian))]
pub fn derive_packet(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    let derive: DeriveInput = syn::parse2(item).unwrap();
    let mut output = TokenStream::new();
    let mut errors = TokenStream::new();

    let ident = derive.ident.clone();
    let fields = match derive.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => fields,
        _ => {
            errors.extend(
                syn::Error::new_spanned(derive, "Packet can only be derived from a struct")
                    .to_compile_error(),
            );
            return errors.into();
        }
    };

    let mut write_code: Vec<TokenStream> = Vec::new();
    let mut read_code: Vec<TokenStream> = Vec::new();

    for field in fields.iter() {
        let mut field_metadata = FieldMetadata::default();
        let field_name = field.ident.clone().unwrap();
        let ty = match &field.ty {
            syn::Type::Path(path) => &path.path.segments,
            _ => panic!("not a path"),
        };

        let field_type_name = &ty.first().unwrap().ident;
        for attr in &field.attrs {
            if attr.path.is_ident("base37") {
                if !ALLOWED_BY_ATTR
                    .get("base37")
                    .unwrap()
                    .contains(&field_type_name.to_string())
                {
                    errors.extend(
                        syn::Error::new_spanned(field, "base37 can only be applied to a String")
                            .to_compile_error(),
                    );
                    break;
                }
                field_metadata.base37 = true;
            }

            if attr.path.is_ident("transform") {
                if !ALLOWED_BY_ATTR
                    .get("transform")
                    .unwrap()
                    .contains(&field_type_name.to_string())
                {
                    errors.extend(
                        syn::Error::new_spanned(
                            field,
                            "transform can only be applied to a bool, u8 or u16",
                        )
                        .to_compile_error(),
                    );
                    break;
                }

                match transform::parse_transform(&attr) {
                    Ok(transform) => {
                        field_metadata.transform = Some(transform);
                    }
                    Err(err) => {
                        errors.extend(err);
                        break;
                    }
                }
            }

            if attr.path.is_ident("endian") {
                if !ALLOWED_BY_ATTR
                    .get("endian")
                    .unwrap()
                    .contains(&field_type_name.to_string())
                {
                    errors.extend(
                        syn::Error::new_spanned(
                            field,
                            "endian can only be applied to a u16 or u32",
                        )
                        .to_compile_error(),
                    );
                    break;
                }

                match endian::parse_endianness(field, &attr) {
                    Ok(endian) => {
                        field_metadata.endian = Some(endian);
                    }
                    Err(err) => {
                        errors.extend(err);
                        break;
                    }
                }
            }
        }

        let write = generate_quote_write(&field_name, field_type_name, &field_metadata);
        let read = generate_quote_read(&field_name, field_type_name, &field_metadata);

        write_code.push(write);
        read_code.push(read);
    }

    let completed = quote! {
        impl crate::packet::Packet for #ident {
            fn try_read(&mut self, src: &mut BytesMut) -> anyhow::Result<()> {
                #(#read_code)*
                Ok(())
            }

            fn try_write(&self, dst: &mut BytesMut) -> anyhow::Result<()> {
                #(#write_code)*
                Ok(())
            }

            fn get_type(&self) -> PacketType {
                PacketType::#ident
            }
        }
    };

    output.extend(completed);
    output.extend(errors);
    output.into()
}
