use std::{cell::{Ref, RefCell, RefMut}, ops::Deref, rc::Rc};

pub type SharableRef<T> = Rc<RefCell<T>>;

pub trait SharableRefExt<T>{
    fn new_sharable(inner: T) -> SharableRef<T>;
    
    fn deref(&self) -> Ref<T>;
        
    fn deref_mut(&mut self) -> RefMut<T>;
}
    
impl<T> SharableRefExt<T> for SharableRef<T>{
    fn new_sharable(inner: T) -> SharableRef<T>{
        Rc::new(RefCell::new(inner))
    }
    fn deref_mut(&mut self) -> RefMut<T> {
        self.borrow_mut()
    }
    fn deref(&self) -> Ref<T>{
        self.borrow()
    }
}