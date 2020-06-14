//! dbin bindings -- for declaratively parsing binary files
//! Stil WIP...
#![allow(dead_code)]
mod context;
mod pat;

use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Opaque;
use crate::RcStr;
use crate::Value;
use context::Context;
// use context::Scope;
use pat::Pattern;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a._dbin";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    for (key, val) in vec![
        ("U8", from_pattern_raw(Pattern::U8)),
        ("I8", from_pattern_raw(Pattern::I8)),
        ("LeU16", from_pattern_raw(Pattern::LeU16)),
        ("LeU32", from_pattern_raw(Pattern::LeU32)),
        ("LeU64", from_pattern_raw(Pattern::LeU64)),
        ("BeU16", from_pattern_raw(Pattern::BeU16)),
        ("BeU32", from_pattern_raw(Pattern::BeU32)),
        ("BeU64", from_pattern_raw(Pattern::BeU64)),
        ("LeI16", from_pattern_raw(Pattern::LeI16)),
        ("LeI32", from_pattern_raw(Pattern::LeI32)),
        ("LeI64", from_pattern_raw(Pattern::LeI64)),
        ("BeI16", from_pattern_raw(Pattern::BeI16)),
        ("BeI32", from_pattern_raw(Pattern::BeI32)),
        ("BeI64", from_pattern_raw(Pattern::BeI64)),
        ("LeF32", from_pattern_raw(Pattern::LeF32)),
        ("LeF64", from_pattern_raw(Pattern::LeF64)),
        ("BeF32", from_pattern_raw(Pattern::BeF32)),
        ("BeF64", from_pattern_raw(Pattern::BeF64)),
        ("CStr", from_pattern_raw(Pattern::CStr)),
    ] {
        map.insert(key.into(), val);
    }

    map.extend(
        vec![
            NativeFunction::simple0(
                sr,
                "pattern_parse",
                &["pat", "bytes"],
                |globals, args, _| {
                    let pattern = expect_pattern(globals, &args[0])?;
                    let bytes = Eval::expect_bytes(globals, &args[1])?;
                    let data = match pattern.parse(globals, &bytes) {
                        Ok(data) => data,
                        Err(error) => return globals.set_exc_str(&format!("{:?}", error)),
                    };
                    Ok(data)
                },
            ),
            NativeFunction::simple0(sr, "new_pattern_exact", &["bytes"], |globals, args, _| {
                let bytes = Eval::expect_bytes_from_pattern(globals, &args[0])?;
                let pat = Pattern::Exact(bytes.into());
                Ok(from_pattern_raw(pat))
            }),
            NativeFunction::simple0(sr, "new_pattern_map", &["pat", "f"], |globals, args, _| {
                let bytes = Eval::expect_bytes_from_pattern(globals, &args[0])?;
                let pat = Pattern::Exact(bytes.into());
                Ok(from_pattern_raw(pat))
            }),
        ]
        .into_iter()
        .map(|f| (f.name().clone(), f.into())),
    );

    Ok({
        let mut ret = HMap::new();
        for (key, value) in map {
            ret.insert(key, Rc::new(RefCell::new(value)));
        }
        ret
    })
}

fn from_pattern_raw(pattern: Pattern) -> Value {
    from_pattern(pattern.into())
}

fn from_pattern(pattern: Rc<Pattern>) -> Value {
    Opaque::new(pattern).into()
}

fn expect_pattern<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, Rc<Pattern>>> {
    Eval::expect_opaque(globals, value)
}
