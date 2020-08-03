//! dbin bindings -- for declaratively parsing binary files
//! Stil WIP...
use rand::rngs::ThreadRng;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Handle;
use crate::RcStr;
use crate::Value;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a._rand";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let mut map = HashMap::<RcStr, Value>::new();

    let rngcls = globals.new_class0("a._rand::Rng", vec![])?;
    globals.set_handle_class::<RngW>(rngcls)?;

    map.extend(
        vec![
            NativeFunction::sdnew(
                "new_rng",
                (&[], &[], None, None),
                Some(concat!("Returns the default RNG")),
                |globals, _args, _| from_thread_rng(globals, rand::thread_rng()).map(From::from),
            ),
            NativeFunction::sdnew(
                "new_rng_seeded",
                (&["seed"], &[], None, None),
                Some(concat!(
                    "Returns a reproducible seeded RNG\n",
                    "The seed may either be a single integer or\n",
                    "a Bytes pattern that resolves to exactly 32 bytes\n",
                )),
                |globals, args, _| {
                    let r = if let Value::Int(i) = &args[0] {
                        ChaCha20Rng::seed_from_u64(*i as u64)
                    } else {
                        let bytes = Eval::expect_bytes_from_pattern(globals, &args[0])?;
                        if bytes.len() != 32 {
                            return globals
                                .set_exc_str(&format!("Seed must provide exactly 32 bytes",));
                        }
                        let mut seed = [0u8; 32];
                        seed.copy_from_slice(&bytes);
                        ChaCha20Rng::from_seed(seed)
                    };
                    from_chacha_rng(globals, r).map(From::from)
                },
            ),
            NativeFunction::sdnew(
                "rng_gen_int",
                (&["rng"], &[], None, None),
                Some(concat!("Generates an Int from the given RNG")),
                |globals, args, _| {
                    let mut r = to_rngw_mut(globals, &args[0])?;
                    Ok(r.gen_int().into())
                },
            ),
            NativeFunction::sdnew(
                "rng_gen_float",
                (&["rng"], &[], None, None),
                Some(concat!("Generates a Float from the given RNG")),
                |globals, args, _| {
                    let mut r = to_rngw_mut(globals, &args[0])?;
                    Ok(r.gen_float().into())
                },
            ),
            NativeFunction::sdnew(
                "rng_gen_int_range",
                (&["rng", "start", "end"], &[], None, None),
                Some(concat!(
                    "Generates an integer in the [start, end) interval from the given RNG"
                )),
                |globals, args, _| {
                    let mut r = to_rngw_mut(globals, &args[0])?;
                    let start = Eval::expect_int(globals, &args[1])?;
                    let end = Eval::expect_int(globals, &args[2])?;
                    Ok(r.gen_int_range(start, end).into())
                },
            ),
            NativeFunction::sdnew(
                "rng_gen_float_range",
                (&["rng", "start", "end"], &[], None, None),
                Some(concat!(
                    "Generates a float in the [start, end) interval from the given RNG"
                )),
                |globals, args, _| {
                    let mut r = to_rngw_mut(globals, &args[0])?;
                    let start = Eval::expect_floatlike(globals, &args[1])?;
                    let end = Eval::expect_floatlike(globals, &args[2])?;
                    Ok(r.gen_float_range(start, end).into())
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

fn from_thread_rng(globals: &mut Globals, tr: ThreadRng) -> EvalResult<Handle<RngW>> {
    let rngw = RngW::ThreadRng(tr);
    globals.new_handle(rngw)
}

fn from_chacha_rng(globals: &mut Globals, tr: ChaCha20Rng) -> EvalResult<Handle<RngW>> {
    let rngw = RngW::ChaCha20Rng(tr);
    globals.new_handle(rngw)
}

fn to_rngw_mut<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<RefMut<'a, RngW>> {
    Eval::handle_borrow_mut(globals, value)
}

enum RngW {
    /// The default RNG to use
    ThreadRng(ThreadRng),

    /// For a reproducible, seedable RNG
    ChaCha20Rng(ChaCha20Rng),
}

impl RngW {
    fn gen_int(&mut self) -> i64 {
        match self {
            RngW::ThreadRng(r) => r.gen(),
            RngW::ChaCha20Rng(r) => r.gen(),
        }
    }
    fn gen_float(&mut self) -> f64 {
        match self {
            RngW::ThreadRng(r) => r.gen(),
            RngW::ChaCha20Rng(r) => r.gen(),
        }
    }
    fn gen_int_range(&mut self, low: i64, high: i64) -> i64 {
        match self {
            RngW::ThreadRng(r) => r.gen_range(low, high),
            RngW::ChaCha20Rng(r) => r.gen_range(low, high),
        }
    }
    fn gen_float_range(&mut self, low: f64, high: f64) -> f64 {
        match self {
            RngW::ThreadRng(r) => r.gen_range(low, high),
            RngW::ChaCha20Rng(r) => r.gen_range(low, high),
        }
    }
}
