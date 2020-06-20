//! ggez wrapper
// mod ctx;
use crate::Globals;
use crate::EvalResult;
use super::add;
use ggez::GameResult;


fn try_<R>(globals: &mut Globals, r: GameResult<R>) -> EvalResult<R> {
    match r {
        Ok(r) => Ok(r),
        Err(error) => globals.set_exc_str(&format!("{:?}", error)),
    }
}

pub(super) fn add_gg_modules(globals: &mut Globals) {
    // add(globals, ctx::NAME, ctx::load);
}
