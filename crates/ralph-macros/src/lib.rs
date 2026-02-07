use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Marks a type for TypeScript export via ts-rs.
///
/// **Structs:** adds `TS` derive, `#[ts(export)]`, and `#[ts(optional_fields)]`.
/// Rename behavior is handled by serde-compat reading existing `#[serde(rename_all)]`.
///
/// **Enums:** adds `TS` derive and `#[ts(export)]` only.
#[proc_macro_attribute]
pub fn ipc_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    let is_struct = matches!(input.data, syn::Data::Struct(_));

    inject_derive(&mut input, "ts_rs", "TS");

    input.attrs.push(parse_attr("#[ts(export)]"));

    if is_struct {
        input.attrs.push(parse_attr("#[ts(optional_fields)]"));
    }

    quote!(#input).into()
}

fn inject_derive(input: &mut DeriveInput, module: &str, name: &str) {
    let module_ident = syn::Ident::new(module, proc_macro2::Span::call_site());
    let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());

    for attr in &mut input.attrs {
        if attr.path().is_ident("derive") {
            let mut found = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(name) {
                    found = true;
                }
                Ok(())
            });
            if found {
                return;
            }
        }
    }

    for attr in &mut input.attrs {
        if attr.path().is_ident("derive") {
            let prev: proc_macro2::TokenStream = attr.parse_args().unwrap();
            *attr = syn::parse_quote!(#[derive(#prev, #module_ident::#name_ident)]);
            return;
        }
    }

    input
        .attrs
        .push(syn::parse_quote!(#[derive(#module_ident::#name_ident)]));
}

fn parse_attr(s: &str) -> syn::Attribute {
    let token_stream: proc_macro2::TokenStream = s.parse().unwrap();
    syn::parse_quote!(#token_stream)
}
