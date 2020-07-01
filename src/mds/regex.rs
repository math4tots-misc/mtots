use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Opaque;
use crate::RcStr;
use crate::Value;
use regex::Match;
use regex::Regex;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a._regex";

// TODO: captures API, and find_all and its variants
pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::simple0(sr, "new_regex", &["pattern"], |globals, args, _| {
                let string = Eval::expect_string(globals, &args[0])?;
                let pattern = match Regex::new(string) {
                    Ok(r) => r,
                    Err(error) => return globals.set_exc_str(&format!("{:?}", error)),
                };
                Ok(from_regex(pattern))
            }),
            NativeFunction::sdnew0(
                sr,
                "regex_find",
                &["pattern", "text", "start", "end"],
                Some(concat!(
                    "Searches for the regex in the given text.\n",
                    "Start and end arguments can optionally be provided to search in ",
                    "a subset of the text.\n",
                    "Returns nil if no match is found, or [start, end] pair denoting the ",
                    "location if it is.\n",
                )),
                |globals, args, _| {
                    let pattern = expect_regex(globals, &args[0])?;
                    let text = Eval::expect_string(globals, &args[1])?;
                    let start = if let Value::Nil = &args[2] {
                        0
                    } else {
                        Eval::expect_usize(globals, &args[2])?
                    };
                    let end = if let Value::Nil = &args[3] {
                        text.len()
                    } else {
                        Eval::expect_usize(globals, &args[3])?
                    };
                    let text = &text[start..end];
                    Ok(match pattern.find(text) {
                        Some(m) => from_match(start, &m),
                        None => Value::Nil,
                    })
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "regex_replace",
                &["pattern", "text", "repl", "start", "end", "limit"],
                Some(concat!(
                    "Returns a new string where all the matching text in the given ",
                    "text is replaced with the given replacement pattern.\n",
                    "The limit (if provided and non-zero) will fix the number of matches ",
                    "that will be replaced.\n",
                    "If the limit is not provided or zero, it will replace all found matches.",
                )),
                |globals, args, _| {
                    let pattern = expect_regex(globals, &args[0])?;
                    let text = Eval::expect_string(globals, &args[1])?;
                    let repl = Eval::expect_string(globals, &args[2])?;
                    let start = if let Value::Nil = &args[3] {
                        0
                    } else {
                        Eval::expect_usize(globals, &args[3])?
                    };
                    let end = if let Value::Nil = &args[4] {
                        text.len()
                    } else {
                        Eval::expect_usize(globals, &args[4])?
                    };
                    let limit = Eval::expect_usize(globals, &args[5])?;
                    let text = &text[start..end];
                    let replaced_string = pattern.replacen(text, limit, repl.str()).into_owned();
                    Ok(replaced_string.into())
                },
            ),
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

fn from_match(string_start: usize, m: &Match) -> Value {
    let start = Value::Int((string_start + m.start()) as i64);
    let end = Value::Int((string_start + m.end()) as i64);
    vec![start, end].into()
}

fn from_regex(r: Regex) -> Value {
    let opaque = Opaque::new(r);
    opaque.into()
}

fn expect_regex<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, Regex>> {
    Eval::expect_opaque(globals, value)
}
