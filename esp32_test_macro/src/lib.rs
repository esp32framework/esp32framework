extern crate proc_macro2;

use std::sync::Mutex;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};
use lazy_static::lazy_static;

lazy_static! {
    static ref FUNC_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

#[proc_macro_attribute]
pub fn esp32_test(_attr: TokenStream, item: TokenStream)-> TokenStream{
    let fn_item = parse_macro_input!(item as ItemFn);

    register_fn_as_test(&fn_item);

    TokenStream::from(quote! {#fn_item})
}

fn register_fn_as_test(func: &ItemFn){

    FUNC_NAMES.lock().unwrap().push(func.sig.ident.to_string()) 
}

#[proc_macro]
pub fn get_all_tests(_item: TokenStream)-> TokenStream{
    let func_names = FUNC_NAMES.lock().unwrap();

    let tests = func_names.iter().map(|name| {
        let fn_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        quote! {
            MyTest { name: #name, func: #fn_ident as fn() }
        }
    });

    let output = quote! {
        #[derive(Debug)]
        struct MyTest {
            name: &'static str,
            func: fn(),
        }

        const ALL_TESTS: &[MyTest] = &[
            #(#tests),*
        ];
    };

    TokenStream::from(output)
}
