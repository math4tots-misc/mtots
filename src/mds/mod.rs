use crate::Globals;

use crate::EvalResult;
use crate::HMap;
use crate::RcStr;
use crate::Value;
use std::cell::RefCell;
use std::rc::Rc;

mod dbin;
mod json;
mod rand;
mod regex;
mod yagl;

pub fn add_standard_modules(globals: &mut Globals) {
    add(globals, dbin::NAME, dbin::load);
    add(globals, json::NAME, json::load);
    add(globals, rand::NAME, rand::load);
    add(globals, regex::NAME, regex::load);
    add(globals, yagl::NAME, yagl::load);
}

fn add<F>(globals: &mut Globals, name: &'static str, body: F)
where
    F: FnOnce(&mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> + 'static,
{
    globals.add_native_module(name.into(), body);
}
