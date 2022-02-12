use proc_macro::TokenStream;
use quote::quote;
use syn::parse;

#[proc_macro_derive(EnumLen)]
pub fn derive_enum_variant_count(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = parse(input).unwrap();

    let len = match syn_item.data {
        syn::Data::Enum(enum_item) => enum_item.variants.len(),
        _ => panic!("EnumVariantCount only works on Enums"),
    };

    let expanded = quote! {
    pub const LEN: usize = #len;
        };
    expanded.into()
}
