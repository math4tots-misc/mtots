//! JSON bindings
use crate::mtry;
use crate::rterr;
use crate::Key;
use crate::List;
use crate::Map;
use crate::NativeModule;
use crate::Result;
use crate::Value;
use std::convert::TryFrom;

pub const NAME: &str = "a.json";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.func(
            "loads",
            ["string"],
            "Takes a json string, and converts it to a mtots value",
            |_globals, args, _| {
                let mut args = args.into_iter();
                let string = args.next().unwrap().into_string()?;
                let serde_value: serde_json::Value = mtry!(serde_json::from_str(&string));
                let value = from_serde(serde_value)?;
                Ok(value)
            },
        );
        m.func(
            "dumps",
            ["blob"],
            "Takes an mtots value and converts it to a json string",
            |_globals, args, _| {
                let mut args = args.into_iter();
                let value = args.next().unwrap();
                let serde_value = to_serde(value)?;
                let string = serde_value.to_string();
                Ok(string.into())
            },
        );
    })
}

pub fn to_serde(value: Value) -> Result<serde_json::Value> {
    match value {
        Value::Invalid => panic!("to_serde(Value::Invalid)"),
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(x) => Ok(serde_json::Value::from(x)),
        Value::Number(x) => {
            if x.is_finite() {
                Ok(if x.fract() == 0.0 {
                    serde_json::Value::from(x as i64)
                } else {
                    serde_json::Value::from(x)
                })
            } else {
                Err(rterr!(
                    "Non-finite numbers cannot be used in JSON ({:?})",
                    x
                ))
            }
        }
        Value::String(x) => Ok(serde_json::Value::String(x.unwrap_or_clone())),
        Value::List(list) => Ok(List::unwrap_or_clone(list)
            .into_iter()
            .map(to_serde)
            .collect::<Result<Vec<_>>>()?
            .into()),
        Value::Map(map) => Ok(serde_json::Value::Object(
            Map::unwrap_or_clone(map)
                .into_iter()
                .map(|(k, v)| Ok((String::try_from(Value::from(k))?, to_serde(v)?)))
                .collect::<Result<serde_json::Map<String, serde_json::Value>>>()?
                .into(),
        )),
        _ => Err(rterr!(
            "{} cannot be converted to JSON",
            value.debug_typename()
        )),
    }
}

pub fn from_serde(value: serde_json::Value) -> Result<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(x) => Ok(Value::from(x)),
        serde_json::Value::Number(x) => Ok(Value::from(x.as_f64().unwrap())),
        serde_json::Value::String(x) => Ok(Value::from(x)),
        serde_json::Value::Array(vals) => Ok(Value::from(
            vals.into_iter()
                .map(from_serde)
                .collect::<Result<Vec<_>>>()?,
        )),
        serde_json::Value::Object(obj) => obj
            .into_iter()
            .map(|(k, v)| Ok((Key::from(k), from_serde(v)?)))
            .collect::<Result<Map>>()
            .map(Value::from),
    }
}
