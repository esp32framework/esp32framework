use std::collections::HashSet;

use syn::{parse_quote, Block, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Signature, Type, TypePath};
use quote::{quote, ToTokens};

#[derive(Debug)]
enum SharableReferenceMacroError{
    SignatureCannotReturnUnreferencedSelf,
    ExpectedTypePath,
    ExpectedSegmentInTypePath
}

trait SharableReferenceImplBlockExt {
    /// Modifies impl block name by removing first character.
    /// 
    /// # Returns
    ///
    /// `Ok(())` uppon sucess, otherwise a `SharableReferenceMacroError`.
    ///
    /// # Errors
    ///
    /// - `SharableReferenceMacroError::ExpectedTypePath`: Upon invalid type.
    /// - `SharableReferenceMacroError::ExpectedSegmentInTypePath`: If there are no segments in path.
    fn remove_first_character(&mut self)-> Result<(), SharableReferenceMacroError>;

    /// Filters and modifies any item of the impl block that needs changing
    fn modify_and_filter_items(&mut self, args: &InnerArgs);
}

trait SharableReferenceImplItemFnExt{
    /// Modifies the signature and body of the ImplItemFn
    /// # Returns
    ///
    /// `Ok(())` on success , otherwise a `SharableReferenceMacroError`.
    ///
    /// # Errors
    ///
    /// - `SharableReferenceMacroError::SignatureCannotReturnUnreferencedSelf`: If Self is being returned, 
    /// since the is no way to return it behind an Rc<RefCell<T>>
    fn modify_for_sharable_ref(&mut self, args: &InnerArgs) -> Result<(),SharableReferenceMacroError>;

    /// Returns wheter or not the ImplItemFn is public
    fn is_public(&self)-> bool;

    /// Returns wheter or not the ImplItemFn receives a reference to self, mutable or not
    fn receives_reference_to_self(&self)->bool;

    /// Returns whether or not this ImplItemFn should be filtered or not
    fn filter_for_sharable_ref(&self)->bool{
        !self.is_public() || !self.receives_reference_to_self()
    }
}

trait SharableReferenceSignatureExt {
    /// Modifies the signature removing any argument in args
    fn modify_for_sharable_ref(&mut self, args: &InnerArgs);

    /// Returns whether or not the signature returns a mutable self
    fn receives_a_mutable_self(&mut self)->bool;

    /// # Returns
    ///
    /// `Ok(bool)` answearing wheter the return type is &self or &mut self , 
    /// otherwise a `SharableReferenceMacroError`.
    ///
    /// # Errors
    ///
    /// - `SharableReferenceMacroError::SignatureCannotReturnUnreferencedSelf`: If Self is being returned, 
    /// since the is no way to return it behind an Rc<RefCell<T>>
    fn outputing_valid_self(&self)->Result<bool, SharableReferenceMacroError>;
}

trait SharableReferenceFnArgExt{
    /// returns whether the argument is contained in args or not
    fn arg_contained_in(&self, args: &InnerArgs) -> bool;
}

impl SharableReferenceImplBlockExt for ItemImpl{
    fn remove_first_character(&mut self) -> Result<(), SharableReferenceMacroError>{
        match self.self_ty.as_mut(){
            syn::Type::Path(type_path) => {
                let segment = type_path.path.segments.last_mut().ok_or(SharableReferenceMacroError::ExpectedSegmentInTypePath)?;
                let new_name = segment.ident.clone().to_string().split_off(1);
                segment.ident = Ident::new(&new_name, segment.ident.span());
                Ok(())
            }
            _ => Err(SharableReferenceMacroError::ExpectedTypePath),
        }
    }

    fn modify_and_filter_items(&mut self, args: &InnerArgs) {
        self.items.retain_mut(|item|
            match item {
                
                ImplItem::Fn(impl_item_fn) => {
                    let filtered = impl_item_fn.filter_for_sharable_ref();
                    if !filtered{
                        impl_item_fn.modify_for_sharable_ref(args);
                    }
                    !filtered
                },
                _ => true,
            }
        );
    }
}

impl SharableReferenceImplItemFnExt for ImplItemFn{
    fn modify_for_sharable_ref(&mut self, args: &InnerArgs) -> Result<(),SharableReferenceMacroError> {
        self.sig.modify_for_sharable_ref(args);
        println!("{}",self.block.to_token_stream());
        
        let borrow = match self.sig.receives_a_mutable_self(){
            true => quote!{self.inner.borrow_mut().},
            false => quote!{self.inner.borrow().}
        };
        
        let final_return_type = match self.sig.outputing_valid_self()?{
            true => quote! { ; self },
            false => quote! { },
        };
        
        let awaiting = match self.sig.asyncness {
            Some(_) => quote! { .await },
            None => quote! {},
        };
        
        let fn_ident = &self.sig.ident;
        
        self.block = parse_quote!({
            #borrow #fn_ident () #awaiting
            #final_return_type
        });

        Ok(())
    }
    
    fn is_public(&self)-> bool {
        match self.vis{
            syn::Visibility::Inherited => false,
            _ => true
        }
    }
    
    fn receives_reference_to_self(&self)->bool {
        self.sig.inputs.iter().any(|arg| match arg{
            FnArg::Receiver(receiver) => receiver.reference.is_some(),
            FnArg::Typed(_) => false,
        })
    }
}

impl SharableReferenceSignatureExt for Signature{
    fn modify_for_sharable_ref(&mut self, args: &InnerArgs) {
        let mut new_args = Vec::new();
        while let Some(pair) = self.inputs.pop(){
            if !pair.value().arg_contained_in(args){
                new_args.push(pair.into_value());
            }
        }
        for new_arg in new_args.into_iter().rev(){
            self.inputs.push(new_arg)
        }
    }
    
    fn receives_a_mutable_self(&mut self)->bool {
        if let Some(arg) = self.inputs.first(){
            if let FnArg::Receiver(r) = arg{
                return r.mutability.is_some()
            }
        }
        false
    }

    fn outputing_valid_self(&self)->Result<bool, SharableReferenceMacroError>{
        if let syn::ReturnType::Type(_, return_type) = &self.output {
            return_type_is_self(return_type.as_ref())?;
        }
        Ok(false)
    }
}

impl SharableReferenceFnArgExt for FnArg{
    fn arg_contained_in(&self, args: &InnerArgs) -> bool {
        if let FnArg::Typed(pat_type) = self{
            let arg_str = pat_type.pat.to_token_stream().to_string();
            return args.contains(arg_str)
        }
        false
    }
}

struct InnerArgs {
    inner: HashSet<String>,
}

impl InnerArgs{
    fn contains(&self, str: String)->bool{
        self.inner.contains(&str)
    }
}

/// # Returns
///
/// `Ok(bool)` answearing wheter the return type is &self or &mut self , 
/// otherwise a `SharableReferenceMacroError`.
///
/// # Errors
///
/// - `SharableReferenceMacroError::SignatureCannotReturnUnreferencedSelf`: If Self is being returned, 
/// since the is no way to return it behind an Rc<RefCell<T>>
fn return_type_is_self(return_type: &Type) -> Result<bool,SharableReferenceMacroError> {
    let res = match return_type {
        Type::Path(type_path) => {
            if type_path_is_self(type_path) {
                return Err(SharableReferenceMacroError::SignatureCannotReturnUnreferencedSelf);
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
    };
    Ok(res)
}

fn type_path_is_self(type_path: &TypePath) -> bool {
    type_path.path.is_ident("Self")
}
/*
pub fn sharable_reference_wrapper(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_original = parse_macro_input!(item as ItemImpl);
    let mut input = input_original.clone();
    let args = parse_macro_input!(args as StringArgs);
    println!("alo");
    input.remove_first_character().unwrap();
    input.modify_and_filter_items();

    //println!("\n\n");
    //println!("{}", input.to_token_stream());
    //println!("\n\n");
    return TokenStream::from(quote! {
        #input_original
        #input
    });
    todo()!
}
*/
