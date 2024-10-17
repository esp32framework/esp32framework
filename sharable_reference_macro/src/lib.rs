extern crate proc_macro2;

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Signature, Type,
    TypePath,
};

/// This macro is used on top of an impl block for `_MyStruct` and creates a new impl block for
/// `MyStruct` with the same methods.
///
/// This new impl block has a method with the same signature for all pub instance methods, while
/// leaving out class methods. This macro assumes `MyStruct` has a field called `inner`, which
/// must be something that implements the methods `borrow()` and `borrow_mut()`, which give a
/// reference or mutable reference respectibly of an instance of `_MyStruct`. For example
/// `inner: Rc<RefCell<_MyStruct>>`.
/// This macro works for:
/// - Generics, as long as they are the same between `_MyStruct` and `MyStruct`.
/// - If any arguments are given, these are interpreted as strings that represent the name of a field
///     of `MyStruct`. If any of the methods receives a parameter with the same name, then the new method
///     wont receive it and instead will use `self.arg`.
/// - Async funtions, since it will add .await to the end.
/// - Function that receive and return &self, or &mut self
///
/// CLARIFICATION: The inner struct does not have to beggin with '_', the macro simply removes the
/// first character from the inner struct to give to the wrapper struct
///
/// # Panics
///
/// This macro will panic if there is a public method on `_MyStruct` that receives and returns self,
/// since the ownership of self is lost on the borrow and cant be returned
///
/// # Example
///
/// ```
/// use sharable_reference_macro::sharable_reference_wrapper;
///
/// struct _Thing<'a, T: Add, const C: i32>{
///     a: u8,
///     b: T,
///     s: &'a str
/// }
///
/// struct Thing<'a, T: Add, const C: i32>{
///     inner: Rc<RefCell<_Thing<'a, T, C>>>,
///     field: u8
/// }
///
/// #[sharable_reference_wrapper("field")]
/// impl<'a, T:Add, const C:i32> _Thing<'a, T, C>{
///     //this method wont get copied since it doesnt receive self
///     fn new(b: T)->Self{
///         Self { a: 1, b , s: "string" }
///     }
///
///     fn private_method(&self){
///         println!("not getting coppied since its a private method")
///     }
///
///     /// This doc will get copied
///     // However comments dont get copied
///     pub fn sum_a_with_number(&self, number: u8) -> u8{
///         self.a + number
///     }
///
///     pub fn sum_a_with_field(&self, field: u8)-> u8{
///         self.a + field
///     }
///
///     pub fn get_s<'b>(&self) -> &'b str
///     where  'a: 'b, {
///         self.s
///     }
///
///     pub(crate) fn add_1_to_a_and_return_ref(&mut self) -> &mut Self{
///         self.a += 1;
///         self
///     }
///
///     pub async fn async_function(&self){
///         //Some async code
///     }
/// }
/// ```
///
/// # Once expanded the following block will be added
///
/// ```
/// impl<'a, T: Add, const C: i32> Thing<'a, T, C> {
///     /// This doc will get copied
///     pub fn sum_a_with_number(&self, number: u8) -> u8 {
///         self.inner.borrow().sum_a_with_number(number)
///     }
///     pub fn sum_a_with_field(&self) -> u8 {
///         self.inner.borrow().sum_a_with_field(self.field)
///     }
///     pub fn get_s<'b>(&self) -> &'b str
///     where
///         'a: 'b,
///     {
///         self.inner.borrow().get_s()
///     }
///     pub(crate) fn add_1_to_a_and_return_ref(&mut self) -> &mut Self {
///         self.inner.borrow_mut().add_1_to_a_and_return_ref();
///         self
///     }
///     pub async fn async_function(&self) {
///         self.inner.borrow().async_function().await
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn sharable_reference_wrapper(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let args = parse_macro_input!(args as StringArgs);
    let (impl_signature, is_trait) = get_impl_signature(&input);
    let new_methods = get_new_items(&input, &args, is_trait);

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

struct StringArgs {
    strings: HashSet<String>,
}

impl Parse for StringArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut strings = HashSet::new();

        while !input.is_empty() {
            // Parse each string literal
            let lit: LitStr = input.parse()?;
            strings.insert(lit.value());
        }

        Ok(StringArgs { strings })
    }
}

/// Returns a vec of all the items of the impl block returning for the methods only the signatures of
/// all public instance methods (those that dont have self) of an impl block. Also if the impl block
/// is a trait returns all of its methods
fn get_new_items(input: &ItemImpl, args: &StringArgs, is_trait: bool) -> Vec<TokenStream2> {
    let mut new_items: Vec<TokenStream2> = Vec::new();
    for item in &input.items {
        match item {
            ImplItem::Fn(method) => {
                if let Some(new_method) = get_pub_instance_method(method, args, is_trait) {
                    new_items.push(new_method)
                }
            }
            _ => new_items.push(item.to_token_stream()),
        }
    }

    new_items
}

fn get_pub_instance_method(
    method: &ImplItemFn,
    args: &StringArgs,
    is_trait: bool,
) -> Option<TokenStream2> {
    let method_attr = &method.attrs;
    let original_sig = &method.sig;
    let method_name = &method.sig.ident;
    let method_visibility = &method.vis;
    let methods_args = original_sig
        .inputs
        .pairs()
        .map(|pair| (*pair.value()).clone());
    let mut method_inputs = vec![];
    let mut is_instance_method = false;
    let mut borrow = quote! {self.inner.borrow().};
    for arg in methods_args {
        match get_inputs_from_arg(arg, &mut borrow, args) {
            Some(input) => method_inputs.push(input),
            None => is_instance_method = true,
        }
    }

    if !is_instance_method {
        return None;
    }

    let method_sig = filter_method_signature(original_sig, args);
    let return_type_has_self = check_if_return_type_ref_self(&method_sig);

    let final_return_type = if return_type_has_self {
        quote! { ; self }
    } else {
        quote! {}
    };

    let awaiting = match method_sig.asyncness {
        Some(_) => quote! { .await },
        None => quote! {},
    };

    let pub_level = match method_visibility {
        syn::Visibility::Public(pub_level) => pub_level.to_token_stream(),
        syn::Visibility::Restricted(restricted) => restricted.to_token_stream(),
        syn::Visibility::Inherited => {
            if is_trait {
                quote! {}
            } else {
                return None;
            }
        }
    };

    Some(quote! {
        #(#method_attr)*
        #pub_level #method_sig {
            #borrow #method_name(#(#method_inputs),*) #awaiting
            #final_return_type
        }
    })
}

/// Returns wether the return type of a signature i &Self of &mut Self.
/// Panics if Self is being returned, since the is no way to return it behind an Rc<RefCell<T>>
fn check_if_return_type_ref_self(sig: &Signature) -> bool {
    if let syn::ReturnType::Type(_, return_type) = &sig.output {
        return_type_is_self(return_type.as_ref())
    } else {
        false
    }
}

/// Returns wether the return type of a signature i &Self of &mut Self.
/// Panics if Self is being returned, since the is no way to return it behind an Rc<RefCell<T>>
fn return_type_is_self(return_type: &Type) -> bool {
    match return_type {
        Type::Path(type_path) => {
            if type_path_is_self(type_path) {
                panic!("Macro does not work for function that return Self, it does however work for &Self, or &mut Self")
            }
            false
        }
        Type::Reference(type_ref) => {
            let refed_type = type_ref.elem.as_ref();
            if let Type::Path(type_path) = refed_type {
                type_path_is_self(type_path)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn type_path_is_self(type_path: &TypePath) -> bool {
    type_path.path.is_ident("Self")
}

/// Returns the input corresponding to each arg. It sets the borro acordingly. If the name of the arg
/// is in args then self.#arg is return if not just #arg.
fn get_inputs_from_arg(
    arg: FnArg,
    borrow: &mut TokenStream2,
    args: &StringArgs,
) -> Option<TokenStream2> {
    match arg {
        syn::FnArg::Receiver(recv) => {
            if recv.mutability.is_some() {
                *borrow = quote! {self.inner.borrow_mut().}
            }
            None
        }
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(id) = &pat_type.pat.as_ref() {
                let arg = &id.ident;
                let arg_str = pat_type.pat.to_token_stream().to_string();
                let method_input = if args.strings.contains(&arg_str) {
                    quote! {self.#arg}
                } else {
                    quote! {#arg}
                };
                return Some(method_input);
            }
            None
        }
    }
}

/// Returns a signature without the methods that have the same name as a string in 'args'
fn filter_method_signature(original_sig: &Signature, args: &StringArgs) -> Signature {
    let mut sig = original_sig.clone();
    sig.inputs = sig
        .inputs
        .into_iter()
        .filter(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                let arg_str = pat_type.pat.to_token_stream().to_string();
                !args.strings.contains(&arg_str)
            } else {
                true
            }
        })
        .collect();
    sig
}

/// Returns a TokenStreams, that represents the new wrapper struct signature, and whether or not is an
/// impl block for a Trait
///  For example:
/// impl <'a, T: Send> _MyStruct <'a,T> will return
/// (impl <'a, T: Send> MyStruct <'a,T>, false)
fn get_impl_signature(input: &ItemImpl) -> (TokenStream2, bool) {
    let wrapper_struct = get_wrapper_struct(input);
    let generics = &input.generics;
    let traits = input.trait_.as_ref().map(|(_, t, _)| t);
    match traits {
        Some(t) => (quote! {impl #generics #t for #wrapper_struct }, true),
        None => (quote! {impl #generics #traits #wrapper_struct }, false),
    }
}

/// Function that returns a TokenStream with the struct of the impl minus the firs character including
/// generics.
fn get_wrapper_struct(input: &ItemImpl) -> TokenStream2 {
    let mut wrapper_struct = input.self_ty.clone();
    match wrapper_struct.as_mut() {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last_mut() {
                let new_name = segment.ident.clone().to_string().split_off(1);

                segment.ident = Ident::new(&new_name, segment.ident.span());
                wrapper_struct.to_token_stream()
            } else {
                panic!("Expected a type path with at least one segment");
            }
        }
        _ => panic!("Expected a path type for the impl"),
    }
}
