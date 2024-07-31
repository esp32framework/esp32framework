extern crate proc_macro2;

use std::collections::HashSet;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, spanned::Spanned, FnArg, GenericParam, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Signature, Token};
    
/// This macro is used on top of an impl block for "_MyStruct" and creates a new impl block for 
/// "MyStruct". This new impl block has a method with the same signature for all pub instance methods,
/// while leaving out class methods. This macro assumes "MyStruct" has a field called "inner", which
/// must be something that implements the trait deref and deref mut which inside has something that 
/// calls "borrow()" and "borrow_mut()", and that inside has an instance of "_MyStruct". For example
/// "inner: Rc<RefCell<_MyStruct>>". This macro works with generics, as long as they are the same 
/// between "_MyStruct" and "MyStruct". 
/// If any arguments are given, these are interpreted as strings, and each represent an aditional field
/// for the Wrapper struct. If any of the methods receives a parameter with the same name, then the
/// new method wont receive it and instead will use "self.arg".
/// CLARIFICATION: The inner struct does not have to beggin with '_', the macro simply removes the 
/// first character from the inner struct to give to the wrapper struct
/// 
/// # Examples
/// 
/// ```
/// #[sharable_reference_wrapper("field")]
/// impl<'a, T: Add, R: Mul, const A: i32> _Cosa<'a, T, R, A> {
///     fn new(x: u8, y: T, z: R, s: &'a str) -> Self {
///         _Cosa {
///             a: x,
///             b: y,
///             c: z,
///             d: s,
///         }
///     }
/// 
///     pub fn sum(&mut self, field: u8) {
///         self.a += field;
///     }
///     pub fn get_a_unmut(&self) -> u8 {
///         self.a
///     }
/// }
/// 
/// //Will generate
/// impl<'a, T: Add, R: Mul, const A: i32> Cosa<'a, T, R, A> {
///     pub fn sum2(&mut self, y: u8) {
///         self.inner.borrow_mut().sum2(self.campo, y)
///     }
///     pub fn get_a_unmut(&self) -> u8 {
///         self.inner.borrow().get_a_unmut()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn sharable_reference_wrapper(args: TokenStream, item: TokenStream) -> TokenStream {
        let input = parse_macro_input!(item as ItemImpl);
        let args = parse_macro_input!(args as StringArgs);
        let (impl_signature, is_trait) = get_impl_signature(&input);
        let new_methods = get_pub_instance_methods(&input, &args, is_trait);

    let new_impl = quote! { 
        #impl_signature{
            #(#new_methods)*
        }
    };
    
    let original_impl = input.to_token_stream();
    
    TokenStream::from(quote! { 
        #original_impl
        #new_impl
    })
}

struct StringArgs{
    strings: HashSet<String>
}

impl Parse for StringArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let mut strings = HashSet::new();

        while !input.is_empty() {
            // Parse each string literal
            let lit: LitStr = input.parse()?;
            strings.insert(lit.value());
        }
        
        Ok(StringArgs {strings})
    }
}

/// Returns a vec of the signatures of all public instance methods (those that dont have self) of an 
/// impl block. Also if the impl block is a trait returns all of its methods
fn get_pub_instance_methods(input :&ItemImpl, args: &StringArgs, is_trait: bool)-> Vec<TokenStream2>{
    let mut new_items: Vec<TokenStream2> = Vec::new();
    for item in &input.items{
        match item{
            ImplItem::Fn(method) => 
                if let Some(new_method) = get_pub_instance_method(method, args, is_trait){
                    new_items.push(new_method)
                },
            _ => new_items.push(item.to_token_stream()),
        }
    }

    new_items
}

fn get_pub_instance_method(method: &ImplItemFn, args: &StringArgs, is_trait: bool)-> Option<TokenStream2>{
    let method_attr = &method.attrs;
    let original_sig = &method.sig;
    let method_name = &method.sig.ident;
    let method_visibility = &method.vis;
    let methods_args = original_sig.inputs.pairs().map( |pair| (*pair.value()).clone());
    let mut method_inputs = vec![];
    let mut is_instance_method = false;
    let mut borrow = quote! {self.inner.borrow().};
    for arg in methods_args{
        match get_inputs_from_arg(arg, &mut borrow, &args){
            Some(input) => method_inputs.push(input),
            None => is_instance_method = true,
        }
    }

    if !is_instance_method{
        return None;
    }

    let method_sig = filter_method_signature(original_sig, &args);

    let pub_level = match method_visibility{
        syn::Visibility::Public(pub_level) => pub_level.to_token_stream(),
        syn::Visibility::Restricted(restricted) => restricted.to_token_stream(),
        syn::Visibility::Inherited => if is_trait{
                quote! {}
            }else{
                return  None;
            }, 
    };

    Some(quote! {
        #(#method_attr)*
        #pub_level #method_sig {
            #borrow #method_name(#(#method_inputs),*)
        }
    })
}

/// Returns the input corresponding to each arg. It sets the borro acordingly. If the name of the arg 
/// is in args then self.#arg is return if not just #arg.
fn get_inputs_from_arg(arg: FnArg, borrow: &mut TokenStream2, args: &StringArgs)->Option<TokenStream2>{
    match arg{
        syn::FnArg::Receiver(recv) => {
            if recv.mutability.is_some(){
                *borrow = quote! {self.inner.borrow_mut().}
            }
            None
        },
        syn::FnArg::Typed(pat_type) => {
            let arg = &pat_type.pat;
            let arg_str = pat_type.pat.to_token_stream().to_string();
            let method_input = if args.strings.contains(&arg_str){
                quote! {self.#arg}
            }else{
                quote! {#arg}
            };
            println!("method_inpu: {}", method_input);
            Some(method_input)
        },
    }
}

/// Returns a signature without the methods that have the same name as a string in 'args'
fn filter_method_signature(original_sig: &Signature, args: &StringArgs)-> Signature{
    let mut sig = original_sig.clone();
    sig.inputs = sig.inputs.into_iter().filter(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            let arg_str = pat_type.pat.to_token_stream().to_string();
            if args.strings.contains(&arg_str){
                false
            }else{
                true
            }
        }else{
            true
        }
        }).collect();
    sig
}

/// Returns a TokenStreams, that represents the new wrapper struct signature, and whether or not is an
/// impl block for a Trait
///  For example: 
/// impl <'a, T: Send> _MyStruct <'a,T> will return 
/// (impl <'a, T: Send> MyStruct <'a,T>, false)
fn get_impl_signature(input: &ItemImpl)-> (TokenStream2, bool){
    let wrapper_struct = get_wrapper_struct(input);
    let generics = &input.generics;
    let traits = input.trait_.as_ref().map(|(_,t,_)| t);
    match traits {
        Some(t) => (quote! {impl #generics #t for #wrapper_struct }, true),
        None => (quote! {impl #generics #traits #wrapper_struct }, false),
    }
}

/// Function that returns a TokenStream with the struct of the impl minus the firs character including
/// generics.
fn get_wrapper_struct(input: &ItemImpl)->TokenStream2{
    let mut wrapper_struct = input.self_ty.clone();
    match wrapper_struct.as_mut() {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last_mut(){
                let new_name = segment.ident.clone().to_string().split_off(1);
                
                segment.ident = Ident::new(&new_name, segment.ident.span());
                return wrapper_struct.to_token_stream()
            } else {
                panic!("Expected a type path with at least one segment");
            }
        }
        _ => panic!("Expected a path type for the impl"),
    }
}