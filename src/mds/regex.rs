use crate::mtry;
use crate::ArgSpec;
use crate::NativeModule;
use crate::RcStr;
use crate::Value;
use regex::Regex;

pub const NAME: &str = "a.regex";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.class::<Regex, _>("Regex", |cls| {
            cls.doc("Binding to Rust/Cargo's 'regex' crate");
            cls.sfunc("__call", ["pattern"], "", |globals, args, _| {
                let mut args = args.into_iter();
                let pattern = args.next().unwrap().into_string()?;
                let re = mtry!(Regex::new(&pattern));
                globals.new_handle::<Regex>(re).map(From::from)
            });
            cls.ifunc(
                "find",
                ArgSpec::builder()
                    .req("text")
                    .def("start", ())
                    .def("end", ()),
                "",
                |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let text = args.next().unwrap().into_string()?;
                    let len = text.len();
                    let start = args.next().unwrap().to_start_index(len)?;
                    let end = args.next().unwrap().to_end_index(len)?;
                    let match_ = owner.borrow().find(&text[start..end]);
                    Ok(match match_ {
                        Some(match_) => globals
                            .new_handle(OwnedMatch {
                                text: text.clone(),
                                start: start + match_.start(),
                                end: start + match_.end(),
                            })?
                            .into(),
                        None => Value::Nil,
                    })
                },
            );
            cls.ifunc(
                "replace",
                ["text", "replacement"],
                "",
                |owner, _globals, args, _| {
                    let mut args = args.into_iter();
                    let text = args.next().unwrap().into_string()?;
                    let replacement = args.next().unwrap().into_string()?;
                    Ok(owner.borrow().replace(&text, replacement.str()).into_owned().into())
                }
            );
        });
        m.class::<OwnedMatch, _>("Match", |cls| {
            cls.str(|match_| match_.str().into());
            cls.repr(|match_| format!("Match({:?})", match_.str()).into());
            cls.ifunc("start", (), "", |owner, _, _, _| {
                Ok(owner.borrow().start.into())
            });
            cls.ifunc("end", (), "", |owner, _, _, _| {
                Ok(owner.borrow().end.into())
            });
            cls.ifunc("str", (), "", |owner, _, _, _| {
                Ok(owner.borrow().str().into())
            });
        });
    })
}

pub struct OwnedMatch {
    text: RcStr,
    start: usize,
    end: usize,
}

impl OwnedMatch {
    pub fn str(&self) -> &str {
        &self.text[self.start..self.end]
    }
}
