//! three bindings
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a._math";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::simple0(sr, "sin", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.sin().into())
            }),
            NativeFunction::simple0(sr, "cos", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.cos().into())
            }),
            NativeFunction::simple0(sr, "tan", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.tan().into())
            }),
            NativeFunction::simple0(sr, "asin", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.asin().into())
            }),
            NativeFunction::simple0(sr, "acos", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.acos().into())
            }),
            NativeFunction::simple0(sr, "atan", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.atan().into())
            }),
            NativeFunction::simple0(sr, "sinh", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.sinh().into())
            }),
            NativeFunction::simple0(sr, "cosh", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.cosh().into())
            }),
            NativeFunction::simple0(sr, "tanh", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.tanh().into())
            }),
            NativeFunction::simple0(sr, "ln", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.ln().into())
            }),
            NativeFunction::simple0(sr, "log2", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.log2().into())
            }),
            NativeFunction::simple0(sr, "log10", &["x"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                Ok(x.log10().into())
            }),
            NativeFunction::simple0(sr, "log", &["x", "base"], |globals, args, _| {
                let x = Eval::expect_floatlike(globals, &args[0])?;
                let y = Eval::expect_floatlike(globals, &args[1])?;
                Ok(x.log(y).into())
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
