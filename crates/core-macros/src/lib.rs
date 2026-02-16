use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, DeriveInput};

/// Marks a type for TypeScript export via ts-rs.
///
/// **Structs:** adds `TS` derive, `#[ts(export)]`, and `#[ts(optional_fields)]`.
/// Rename behavior is handled by serde-compat reading existing `#[serde(rename_all)]`.
///
/// **Enums:** adds `TS` derive and `#[ts(export)]` only.
#[proc_macro_attribute]
pub fn ipc_type(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    let is_struct = matches!(input.data, syn::Data::Struct(_));
    let is_enum = matches!(input.data, syn::Data::Enum(_));

    if let Some(attr) = input.attrs.iter().find(|a| a.path().is_ident("serde")) {
        return syn::Error::new_spanned(
            attr,
            "container-level #[serde(...)] is forbidden on #[ipc_type] types; move config into #[ipc_type(...)]",
        )
        .to_compile_error()
        .into();
    }

    let args = match IpcTypeArgs::parse(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    // Common derives: we inject fully-qualified serde paths so call sites don't need imports.
    let mut required_derives: Vec<syn::Path> = vec![
        syn::parse_quote!(Debug),
        syn::parse_quote!(Clone),
        syn::parse_quote!(serde::Serialize),
    ];

    if !(args.serialize_only || args.custom_deserialize) {
        required_derives.push(syn::parse_quote!(serde::Deserialize));
    }

    required_derives.push(syn::parse_quote!(ts_rs::TS));
    ensure_derives(&mut input, &required_derives);

    input.attrs.push(parse_attr("#[ts(export)]"));

    if is_struct {
        input.attrs.push(parse_attr("#[ts(optional_fields)]"));
    }

    let rename_all = args.rename_all.clone().unwrap_or_else(|| {
        if is_struct {
            "camelCase".to_owned()
        } else if is_enum {
            "snake_case".to_owned()
        } else {
            // Unions are not supported by this macro today.
            "camelCase".to_owned()
        }
    });

    let include_deny_unknown_fields =
        is_struct && !(args.serialize_only || args.custom_deserialize || args.allow_unknown_fields);

    if include_deny_unknown_fields {
        input.attrs.push(syn::parse_quote!(
            #[serde(rename_all = #rename_all, deny_unknown_fields)]
        ));
    } else {
        input
            .attrs
            .push(syn::parse_quote!(#[serde(rename_all = #rename_all)]));
    }

    quote!(#input).into()
}

#[derive(Default, Debug, Clone)]
struct IpcTypeArgs {
    rename_all: Option<String>,
    serialize_only: bool,
    custom_deserialize: bool,
    allow_unknown_fields: bool,
}

impl IpcTypeArgs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let ts: proc_macro2::TokenStream = attr.into();
        if ts.is_empty() {
            return Ok(Self::default());
        }

        let mut out = Self::default();

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("rename_all") {
                let val: syn::LitStr = meta.value()?.parse()?;
                if val.value().is_empty() {
                    return Err(meta.error("rename_all must be a non-empty string"));
                }
                out.rename_all = Some(val.value());
                return Ok(());
            }

            if meta.path.is_ident("serialize_only") {
                out.serialize_only = true;
                return Ok(());
            }

            if meta.path.is_ident("custom_deserialize") {
                out.custom_deserialize = true;
                return Ok(());
            }

            if meta.path.is_ident("allow_unknown_fields") {
                out.allow_unknown_fields = true;
                return Ok(());
            }

            Err(meta.error(
                "unknown #[ipc_type(...)] argument; expected rename_all = \"...\", serialize_only, custom_deserialize, or allow_unknown_fields",
            ))
        });

        parser.parse2(ts)?;

        if out.serialize_only && out.custom_deserialize {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "serialize_only and custom_deserialize are mutually exclusive",
            ));
        }

        Ok(out)
    }
}

fn ensure_derives(input: &mut DeriveInput, required: &[syn::Path]) {
    // Find the first `#[derive(...)]` and extend it; otherwise create a new one.
    let mut derive_attr_idx: Option<usize> = None;
    for (i, attr) in input.attrs.iter().enumerate() {
        if attr.path().is_ident("derive") {
            derive_attr_idx = Some(i);
            break;
        }
    }

    let mut derives: Vec<syn::Path> = Vec::new();
    if let Some(i) = derive_attr_idx {
        let attr = &input.attrs[i];
        let parsed: syn::punctuated::Punctuated<syn::Path, syn::Token![,]> = attr
            .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
            .unwrap();
        derives.extend(parsed);
    }

    let mut has: std::collections::BTreeSet<String> = derives
        .iter()
        .filter_map(|p| p.segments.last().map(|s| s.ident.to_string()))
        .collect();

    for req in required {
        let key = req
            .segments
            .last()
            .expect("required derive path must be non-empty")
            .ident
            .to_string();
        if has.insert(key) {
            derives.push(req.clone());
        }
    }

    let new_attr: syn::Attribute = syn::parse_quote!(#[derive(#(#derives),*)]);

    match derive_attr_idx {
        Some(i) => input.attrs[i] = new_attr,
        None => input.attrs.push(new_attr),
    }
}

fn parse_attr(s: &str) -> syn::Attribute {
    let token_stream: proc_macro2::TokenStream = s.parse().unwrap();
    syn::parse_quote!(#token_stream)
}
