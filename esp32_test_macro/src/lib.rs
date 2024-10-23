extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn esp32_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(item as ItemFn);

    into_test_case(&fn_item)
}

// #[derive(Debug)]
// struct TestFunction {
//     name: &'static str,
//     func: fn(),
// }

fn into_test_case(func: &ItemFn) -> TokenStream {
    let ident = func.sig.ident.clone();
    let name = ident.to_string();
    let aux_test_fn = syn::Ident::new(&format!("test_{name}"), proc_macro2::Span::call_site());

    TokenStream::from(quote! {
        #func

        #[test_case]
        const #aux_test_fn :(&'static str, fn()) = ( #name,  #ident as fn());
    })
}
