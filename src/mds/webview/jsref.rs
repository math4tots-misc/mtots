use super::WV;
use crate::rterr;
use crate::Handle;
use crate::Result;
use crate::WeakHandle;
use std::ops::Drop;

pub(super) struct JsRef {
    pub(super) wv: WeakHandle<WV>,
    pub(super) id: usize,
}

impl Drop for JsRef {
    fn drop(&mut self) {
        if let Some(wv) = self.wv.upgrade() {
            wv.borrow_mut()
                .0
                .eval(&format!("delete $$REFS.a{}", self.id))
                .unwrap();
        }
    }
}

impl JsRef {
    pub fn wv(&self) -> Result<Handle<WV>> {
        match self.wv.upgrade() {
            Some(wv) => Ok(wv),
            None => Err(rterr!("Webview no longer exists")),
        }
    }
}
