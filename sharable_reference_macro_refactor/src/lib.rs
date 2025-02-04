use syn::{FnArg, Ident, ImplItem, ImplItemFn, ItemImpl};

#[derive(Debug)]
enum SharableReferenceMacroError{
    ExpectedTypePath,
    ExpectedSegmentInTypePath
}

trait SharableReferenceImplItemFnExt{
    fn modify_for_sharable_ref(&mut self);
    /// Returns wheter or not the ImplItemFn is public
    fn is_public(&self)-> bool;
    /// Returns wheter or not the ImplItemFn receives a reference to self, mutable or not
    fn receives_reference_to_self(&self)->bool;
    /// Returns whether or not this ImplItemFn should be filtered or not
    fn filter_for_sharable_ref(&self)->bool{
        !self.is_public() || !self.receives_reference_to_self()
    }
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
    fn modify_and_filter_items(&mut self);
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

    fn modify_and_filter_items(&mut self) {
        self.items.retain_mut(|item|
            match item {
                ImplItem::Fn(impl_item_fn) => {
                    impl_item_fn.modify_for_sharable_ref();
                    !impl_item_fn.filter_for_sharable_ref()
                },
                _ => true,
            }
        );
    }
}

impl SharableReferenceImplItemFnExt for ImplItemFn{
    fn modify_for_sharable_ref(&mut self) {
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
