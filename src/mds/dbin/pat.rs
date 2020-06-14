use super::Context;
// use super::Scope;
use crate::EvalResult;
use crate::Globals;
use crate::Symbol;
use crate::Value;
use crate::Eval;
use std::rc::Rc;

pub enum Pattern {
    Exact(Rc<Vec<u8>>), // expect an exact sequence of bytes

    // integral types
    U8,
    I8,
    LeU16,
    LeU32,
    LeU64,
    BeU16,
    BeU32,
    BeU64,
    LeI16,
    LeI32,
    LeI64,
    BeI16,
    BeI32,
    BeI64,

    // float types
    LeF32,
    LeF64,
    BeF32,
    BeF64,

    // null terminated string
    CStr,

    // Array, with variable length
    Array(Rc<Pattern>, Value),

    AnyOf(Vec<Pattern>),
    AllOf(Vec<Pattern>), // results in Seq of patterns

    // pseudo patterns
    // these change the parsed results and parse state,
    // but do not directly modify what sequence of bytes
    // they match
    Store(Rc<Pattern>, Symbol), // stores the resulting Value into the current scope
    Map(Rc<Pattern>, Value),
}

impl Pattern {
    pub fn map(
        pat: Rc<Pattern>,
        f: &Value,
    ) -> Pattern {
        Pattern::Map(pat, f.clone())
    }
    pub fn store<K: Into<Symbol>>(pat: Rc<Pattern>, key: K) -> Pattern {
        Pattern::Store(pat, key.into())
    }
    pub fn parse(&self, globals: &mut Globals, bytes: &[u8]) -> EvalResult<Value> {
        let mut ctx = Context::new(globals, bytes);
        self.parse_ctx(&mut ctx)
    }
    fn parse_ctx(&self, ctx: &mut Context) -> EvalResult<Value> {
        match self {
            Pattern::Exact(bytes) => {
                let peek = ctx.peek(bytes.len())?;
                if bytes.as_slice() == peek {
                    Ok(Value::Bytes(ctx.read(bytes.len())?.to_vec().into()))
                } else {
                    ctx.err(format!("Expected {:?} but got {:?}", bytes, peek))
                }
            }
            Pattern::U8 => Ok((uint(true, ctx.read(1)?) as i64).into()),
            Pattern::I8 => Ok((sint(true, ctx.read(1)?) as i64).into()),
            Pattern::LeU16 => Ok((uint(true, ctx.read(2)?) as i64).into()),
            Pattern::LeU32 => Ok((uint(true, ctx.read(4)?) as i64).into()),
            Pattern::LeU64 => Ok((uint(true, ctx.read(8)?) as i64).into()),
            Pattern::BeU16 => Ok((uint(false, ctx.read(2)?) as i64).into()),
            Pattern::BeU32 => Ok((uint(false, ctx.read(4)?) as i64).into()),
            Pattern::BeU64 => Ok((uint(false, ctx.read(8)?) as i64).into()),
            Pattern::LeI16 => Ok((sint(true, ctx.read(2)?) as i64).into()),
            Pattern::LeI32 => Ok((sint(true, ctx.read(4)?) as i64).into()),
            Pattern::LeI64 => Ok((sint(true, ctx.read(8)?) as i64).into()),
            Pattern::BeI16 => Ok((sint(false, ctx.read(2)?) as i64).into()),
            Pattern::BeI32 => Ok((sint(false, ctx.read(4)?) as i64).into()),
            Pattern::BeI64 => Ok((sint(false, ctx.read(8)?) as i64).into()),
            Pattern::LeF32 => Ok((f32::from_bits(uint(true, ctx.read(4)?) as u32) as f64).into()),
            Pattern::LeF64 => Ok((f64::from_bits(uint(true, ctx.read(8)?) as u64)).into()),
            Pattern::BeF32 => Ok((f32::from_bits(uint(false, ctx.read(4)?) as u32) as f64).into()),
            Pattern::BeF64 => Ok((f64::from_bits(uint(false, ctx.read(8)?) as u64)).into()),
            Pattern::CStr => {
                let mut bytes = Vec::new();
                while ctx.peek(1)?[0] != 0 {
                    bytes.push(ctx.read(1)?[0]);
                }
                match std::str::from_utf8(&bytes) {
                    Ok(s) => Ok(s.into()),
                    Err(error) => ctx.err(format!("{:?}", error)),
                }
            }
            Pattern::Array(pat, expr) => {
                let (globals, _scope) = ctx.globals_mut_and_scope();
                let value = Eval::call(globals, expr, vec![])?;
                let len = match value {
                    Value::Int(i) => i as usize,
                    x => return ctx.err(format!("Got non-int for array len ({:?})", x)),
                };
                let mut ret = Vec::new();
                for _ in 0..len {
                    ret.push(pat.parse_ctx(ctx)?);
                }
                Ok(ret.into())
            }
            Pattern::AnyOf(pats) => {
                let pos = ctx.save();
                let mut last = ctx.err("Empty 'any-of'");
                for pat in pats {
                    last = pat.parse_ctx(ctx);
                    if last.is_ok() {
                        return last;
                    } else {
                        ctx.restore(pos);
                    }
                }
                last
            }
            Pattern::AllOf(pats) => {
                let mut ret = Vec::new();
                for pat in pats {
                    ret.push(pat.parse_ctx(ctx)?);
                }
                Ok(ret.into())
            }
            Pattern::Store(pat, key) => {
                let val = pat.parse_ctx(ctx)?;
                ctx.scope_mut().set(*key, val.clone());
                Ok(val)
            }
            Pattern::Map(pat, _f) => {
                let _val = pat.parse_ctx(ctx)?;
                let (_globals, _scope) = ctx.globals_mut_and_scope();
                panic!("TODO")
            }
        }
    }
}

fn uint(little_endian: bool, bytes: &[u8]) -> u64 {
    let mut ret: u64 = 0;
    if little_endian {
        for byte in bytes.iter().rev() {
            ret <<= 8;
            ret += (*byte) as u64;
        }
    } else {
        for byte in bytes {
            ret <<= 8;
            ret += (*byte) as u64;
        }
    }
    ret
}

fn sint(little_endian: bool, bytes: &[u8]) -> i64 {
    let mut bytes = bytes.to_vec();
    let byte = if little_endian {
        *bytes.last_mut().unwrap() as i8
    } else {
        bytes[0] as i8
    };
    let minus = if byte < 0 {
        for byte in &mut bytes {
            *byte = !*byte;
        }
        true
    } else {
        false
    };
    let ui = uint(little_endian, &bytes);
    if minus {
        -(ui.wrapping_add(1) as i64)
    } else {
        ui as i64
    }
}
