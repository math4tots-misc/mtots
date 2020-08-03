//! JSON bindings
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Symbol;
use crate::VMap;
use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a.json";

pub(super) fn load(_globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::simple0("loads", &["string"], |globals, args, _| {
                let string = Eval::expect_string(globals, &args[0])?;
                let serde_value: serde_json::Value = match serde_json::from_str(string) {
                    Ok(value) => value,
                    Err(error) => {
                        return globals.set_exc_str(&format!("{:?}", error));
                    }
                };
                let value = serde_to_mtots(globals, serde_value)?;
                Ok(value)
            }),
            NativeFunction::simple0("dumps", &["blob"], |globals, args, _| {
                let serde_value = mtots_to_serde(globals, &args[0])?;
                let string = serde_value.to_string();
                Ok(string.into())
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

fn serde_to_mtots(globals: &mut Globals, sv: serde_json::Value) -> EvalResult<Value> {
    Ok(match sv {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(x) => Value::Bool(x),
        serde_json::Value::Number(x) => {
            if let Some(x) = x.as_i64() {
                Value::Int(x)
            } else if let Some(x) = x.as_u64() {
                Value::Float(x as f64)
            } else if let Some(x) = x.as_f64() {
                Value::Float(x)
            } else {
                panic!("serde_json Number to mtots::Value")
            }
        }
        serde_json::Value::String(x) => x.into(),
        serde_json::Value::Array(arr) => {
            let mut out = Vec::new();
            for x in arr.into_iter() {
                out.push(serde_to_mtots(globals, x)?);
            }
            Value::List(out.into())
        }
        serde_json::Value::Object(obj) => {
            // We use a Map instead of a Table, because Maps preserve order
            let mut map = VMap::new();
            for (key, val) in obj {
                let key = Symbol::from(&key);
                let val = serde_to_mtots(globals, val)?;
                map.s_insert(globals, key.into(), val)?;
            }
            Value::Map(map.into())
        }
    })
}

fn mtots_to_serde(globals: &mut Globals, val: &Value) -> EvalResult<serde_json::Value> {
    Ok(match val {
        Value::Nil => serde_json::Value::Null,
        Value::Bool(x) => serde_json::Value::Bool(*x),
        Value::Int(x) => serde_json::Value::Number((*x).into()),
        Value::Float(x) => serde_json::Value::Number(match serde_json::Number::from_f64(*x) {
            Some(x) => x,
            None => {
                return globals.set_exc_str(&format!("Float value is not valid for JSON ({})", x));
            }
        }),
        Value::String(s) => serde_json::Value::String(s.str().to_owned()),
        Value::List(arr) => {
            let mut ret = Vec::new();
            for x in arr.iter() {
                ret.push(mtots_to_serde(globals, x)?);
            }
            serde_json::Value::Array(ret)
        }
        Value::Table(_) => globals.set_exc_str(concat!(
            "Tables cannot be converted to json because they do not preserve order ",
            "(use Maps with Symbol keys instead)"
        ))?,
        Value::Map(map) => {
            let mut ret = serde_json::Map::new();
            for (key, val) in map.iter() {
                let key = Eval::expect_symbol(globals, key)?;
                let val = mtots_to_serde(globals, val)?;
                ret.insert(key.str().to_owned(), val);
            }
            serde_json::Value::Object(ret)
        }
        _ => {
            let cls = Eval::classof(globals, val)?;
            let clsname = cls.full_name().clone();
            globals.set_exc_str(&format!(
                "Cannot convert value of type {} into json",
                clsname,
            ))?
        }
    })
}
