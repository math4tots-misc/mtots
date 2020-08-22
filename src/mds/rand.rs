//! Random number generator bindings
use crate::ArgSpec;
use crate::NativeModule;
use crate::Value;
use rand::distributions::uniform::SampleBorrow;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

pub const NAME: &str = "a.rand";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.class::<RngW, _>("Rng", |cls| {
            cls.sfunc(
                "__call",
                ArgSpec::builder().def("seed", ()),
                "",
                |globals, args, _| {
                    let mut args = args.into_iter();
                    let rng = match args.next().unwrap() {
                        Value::Nil => RngW::ThreadRng(rand::thread_rng()),
                        value => {
                            let seed = value.number()?.to_bits();
                            RngW::ChaCha20Rng(ChaCha20Rng::seed_from_u64(seed))
                        }
                    };
                    globals.new_handle(rng).map(From::from)
                },
            );
            cls.ifunc(
                "float",
                ArgSpec::builder().def("low", ()).def("high", ()),
                "",
                |owner, _globals, args, _| {
                    if args[0].is_nil() {
                        let x: f64 = owner.borrow_mut().gen();
                        Ok(Value::from(x))
                    } else {
                        let mut args = args.into_iter();
                        let low = args.next().unwrap().number()?;
                        let high = args.next().unwrap().number()?;
                        let x: f64 = owner.borrow_mut().gen_range(low, high);
                        Ok(Value::from(x))
                    }
                },
            );
            cls.ifunc(
                "int",
                ArgSpec::builder().def("low", ()).def("high", ()),
                "",
                |owner, _globals, args, _| {
                    if args[0].is_nil() {
                        let x: i64 = owner.borrow_mut().gen();
                        Ok(Value::from(x))
                    } else {
                        let mut args = args.into_iter();
                        let low = args.next().unwrap().i64()?;
                        let high = args.next().unwrap().i64()?;
                        let x: i64 = owner.borrow_mut().gen_range(low, high);
                        Ok(Value::from(x))
                    }
                },
            );
            cls.ifunc("choose", ["list"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let list = args.next().unwrap().into_list()?;
                let list = list.borrow();
                let len = list.len();
                let index = owner.borrow_mut().gen_range(0, len);
                Ok(list[index].clone())
            });
        });
        m.func(
            "float",
            ArgSpec::builder().def("low", ()).def("high", ()),
            "",
            |_globals, args, _| {
                let mut owner = rand::thread_rng();
                if args[0].is_nil() {
                    let x: f64 = owner.gen();
                    Ok(Value::from(x))
                } else {
                    let mut args = args.into_iter();
                    let low = args.next().unwrap().number()?;
                    let high = args.next().unwrap().number()?;
                    let x: f64 = owner.gen_range(low, high);
                    Ok(Value::from(x))
                }
            },
        );
        m.func(
            "int",
            ArgSpec::builder().def("low", ()).def("high", ()),
            "",
            |_globals, args, _| {
                let mut owner = rand::thread_rng();
                if args[0].is_nil() {
                    let x: i64 = owner.gen();
                    Ok(Value::from(x))
                } else {
                    let mut args = args.into_iter();
                    let low = args.next().unwrap().i64()?;
                    let high = args.next().unwrap().i64()?;
                    let x: i64 = owner.gen_range(low, high);
                    Ok(Value::from(x))
                }
            },
        );
    })
}

enum RngW {
    /// The default RNG to use
    ThreadRng(ThreadRng),

    /// For a reproducible, seedable RNG
    ChaCha20Rng(ChaCha20Rng),
}

impl RngW {
    fn gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        match self {
            RngW::ThreadRng(r) => r.gen(),
            RngW::ChaCha20Rng(r) => r.gen(),
        }
    }
    fn gen_range<T: SampleUniform, B1, B2>(&mut self, low: B1, high: B2) -> T
    where
        B1: SampleBorrow<T> + Sized,
        B2: SampleBorrow<T> + Sized,
    {
        match self {
            RngW::ThreadRng(r) => r.gen_range(low, high),
            RngW::ChaCha20Rng(r) => r.gen_range(low, high),
        }
    }
}
