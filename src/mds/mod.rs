use crate::Globals;

use crate::EvalResult;
use crate::HMap;
use crate::RcStr;
use crate::Value;
use std::cell::RefCell;
use std::rc::Rc;

mod ggez;
mod json;
mod kiss3d;
mod mfs;
mod time;

pub fn add_standard_modules(globals: &mut Globals) {
    add(globals, json::NAME, json::load);
    add(globals, mfs::NAME, mfs::load);
    add(globals, time::NAME, time::load);
    add(globals, kiss3d::NAME, kiss3d::load);
    add(globals, ggez::NAME, ggez::load);
}

fn add<F>(globals: &mut Globals, name: &'static str, body: F)
where
    F: FnOnce(&mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> + 'static,
{
    globals.add_native_module(name.into(), body);
}
