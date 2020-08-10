use crate::mtry;
use crate::NativeModule;
use regex::Regex;

pub const NAME: &str = "a.regex";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.class::<Regex, _>("Regex", |cls| {
            cls.sfunc("__call", ["pattern"], "", |globals, args, _| {
                let mut args = args.into_iter();
                let pattern = args.next().unwrap().into_string()?;
                let re = mtry!(Regex::new(&pattern));
                globals.new_handle::<Regex>(re).map(From::from)
            });
        });
    })
}
