use crate::content::unwrap_inner_ref;
use std::cell::{Ref, RefCell};

#[derive(Debug)]
pub(crate) struct ResetableOnceCell<T> {
    inner: RefCell<Option<T>>,
}

impl<T> ResetableOnceCell<T> {
    pub(crate) fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    pub(crate) fn get_or_init<F>(&self, f: F) -> Ref<'_, T>
    where
        F: FnOnce() -> T,
    {
        let inner = self.inner.borrow();
        if inner.is_some() {
            return unwrap_inner_ref(inner);
        }

        let value = f();

        *self.inner.borrow_mut() = Some(value);

        unwrap_inner_ref(self.inner.borrow())
    }

    pub(crate) fn reset(&self) {
        self.inner.borrow_mut().take();
    }
}
