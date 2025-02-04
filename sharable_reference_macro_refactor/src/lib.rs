use syn::{Ident, ImplItem, ImplItemFn, ItemImpl};

#[derive(Debug)]
enum SharableReferenceMacroError{
    ExpectedTypePath,
    ExpectedSegmentInTypePath
}

trait SharableReferenceImplBlockExt {
    fn remove_first_character(&mut self)-> Result<(), SharableReferenceMacroError>;
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
}

