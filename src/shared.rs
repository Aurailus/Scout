// Adapted from https://gist.github.com/stevedonovan/7e3a6d8c8921e3eff16c4b11ab82b8d7

use std::fmt;
use std::rc::Rc;
use std::ops::Deref;
use std::cell::{ Ref, RefCell, RefMut, /*BorrowError,*/ BorrowMutError };

#[derive(Clone)]
pub struct Shared<T> {
	v: Rc<RefCell<T>>
}

impl <T> Shared<T> {
	pub fn new(t: T) -> Shared<T> {
		Shared { v: Rc::new(RefCell::new(t)) }
	}
}

impl <T> Shared<T> {
	pub fn borrow(&self) -> Ref<T> {
		self.v.borrow()
	}

	// pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError> {
	// 	self.v.try_borrow()
	// }

	pub fn borrow_mut(&self) -> RefMut<T> {
		self.v.borrow_mut()
	}

	pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
		self.v.try_borrow_mut()
	}

	// pub fn as_ptr(&self) -> *mut T {
	// 	self.v.as_ptr()
	// }

	pub fn clone(&self) -> Shared<T> {
		Shared { v: Rc::clone(&self.v) }
	}
}


impl <T: fmt::Display> fmt::Display for Shared<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.deref())
	}
}


impl <T: fmt::Debug> fmt::Debug for Shared<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.deref())
	}
}
