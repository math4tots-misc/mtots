//! I needed to somehow expose the 'Context' variable to the script itself
//! The problem is that to wrap something in a Value, it needs to be 'static,
//! but the ggez Context is only ever available as a &mut.
//! I couldn't figure out a way to do it without any unsafe code, but at least
//! I think I can limit the unsafe to just this module.
use crate::ggez;
use crate::ggez::Context;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::Opaque;
use crate::Value;
use std::cell::Ref;
use std::cell::RefMut;
use std::rc::Rc;

/// Struct that wraps around a raw &'static mut Context.
/// The context isn't actually 'static, but to be able to
/// store it in a Value, it needs to be 'static.
///
/// By carefully restricting where WrappedContext can ever be
/// constructed, and the way that the outside world can interact
/// with the Context, we ensure that Context is never used
/// improperly
///
/// TODO: audit this
///
pub(super) struct WrappedContext {
    ctx: &'static mut Context,
}

impl WrappedContext {
    pub fn get(&self) -> &Context {
        self.ctx
    }
    pub fn quit(&mut self) {
        ggez::event::quit(self.ctx);
    }
}

pub(super) fn to_wctx<'a>(
    globals: &mut Globals,
    v: &'a Value,
) -> EvalResult<Ref<'a, WrappedContext>> {
    Eval::expect_opaque(globals, v)
}

pub(super) fn to_wctx_mut<'a>(
    globals: &mut Globals,
    v: &'a Value,
) -> EvalResult<RefMut<'a, WrappedContext>> {
    Eval::expect_opaque_mut(globals, v)
}

pub(super) fn with_wctx<F, R>(globals: &mut Globals, ctx: &mut Context, f: F) -> EvalResult<R>
where
    F: FnOnce(&mut Globals, Value) -> EvalResult<R>,
{
    let wctx = WrappedContext {
        ctx: unsafe { std::mem::transmute::<&'_ mut Context, &'static mut Context>(ctx) },
    };
    let opaque_rc = Rc::new(Opaque::new(wctx));
    let result = {
        let value = Value::Opaque(opaque_rc.clone());
        let result = f(globals, value);
        result
    };

    // we check that we have the last reference to opaque_rc.
    // if there are more references, then the wctx object has escaped
    // if that happens, while the code up to here should still be safe
    // (ctx is guaranteed to outlive this call), it would be no longer
    // safe to proceed after this function ends
    if let Ok(_) = Rc::try_unwrap(opaque_rc) {
        // The WrappedContext should be properly disposed of after this block
    } else {
        panic!("ggez::Context has escaped, it is no longer safe to continue");
    }

    result
}
