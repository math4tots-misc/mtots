//! ggez wrapper
//! Lots of the documentation are lifted from ggez's own docs
mod ev;
mod gamepad;
mod gr;
mod wctx;
use super::add;
use crate::EvalResult;
use crate::Globals;
use crate::Stashable;
use gamepad::GamepadRegistry;
use ggez::GameError;
use ggez::GameResult;
use wctx::to_wctx;
use wctx::with_wctx;

pub(super) fn add_gg_modules(globals: &mut Globals) {
    add(globals, gr::NAME, gr::load);
    add(globals, ev::NAME, ev::load);
}

fn try_<R>(globals: &mut Globals, r: GameResult<R>) -> EvalResult<R> {
    match r {
        Ok(r) => Ok(r),
        Err(error) => globals.set_exc_str(&format!("{:?}", error)),
    }
}

#[allow(dead_code)]
fn to_game_result<R>(eval_result: EvalResult<R>) -> GameResult<R> {
    match eval_result {
        Ok(r) => Ok(r),
        Err(_) => Err(GameError::EventLoopError("script error".to_owned())),
    }
}

/// Shared global state between all the native gg modules
struct Shared {
    pub gamepad: GamepadRegistry,
}

impl Stashable for Shared {}
impl Default for Shared {
    fn default() -> Self {
        Shared {
            gamepad: GamepadRegistry::new(),
        }
    }
}
