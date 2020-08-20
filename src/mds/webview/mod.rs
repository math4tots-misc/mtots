//! JSON bindings
use crate::mtry;
use crate::ordie;
use crate::rterr;
use crate::ArgSpec;
use crate::Globals;
use crate::NativeModule;
use crate::Promise;
use crate::Result;
use crate::Value;
use crate::WeakHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

mod jsref;

use jsref::*;

pub const NAME: &str = "a.webview";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        let reg = Rc::new(RefCell::new(JsRequestRegistry::default()));

        m.val(
            "JS_PRELUDE",
            "Little JS stub needed for some of the binding features to work",
            r##"'use strict';
            const $$REFS = Object.create(null);
            var $$REFID = 0;
            function $$RETJSON(reqId, x) {
                const json = JSON.stringify(x);
                external.invoke('eval/' + reqId + '/1/' + (json === undefined ? null : json));
            }
            function $$RETREF(reqId, x) {
                const refId = $$REFID++;
                $$REFS['a' + refId] = x;
                external.invoke('eval/' + reqId + '/2/' + refId);
            }
            function $$ERR(reqId, e) {
                external.invoke('eval/' + reqId + '/3/' + e);
            }
            "##,
        );

        let reg_for_init = reg.clone();
        m.func(
            "init",
            ArgSpec::builder()
                .def("title", "some-window")
                .def("content", "<html></html>")
                .def("size", [800, 600])
                .def("resizable", false)
                .def("debug", false)
                .def("frameless", false)
                .def("handler", ()),
            "",
            move |globals, args, _| {
                let mut args = args.into_iter();
                let title = args.next().unwrap().into_string()?;
                let content = args.next().unwrap().into_string()?;
                let [width, height] = <[i32; 2]>::try_from(args.next().unwrap())?;
                let resizable = args.next().unwrap().truthy();
                let debug = args.next().unwrap().truthy();
                let frameless = args.next().unwrap().truthy();
                let handler = args.next().unwrap();

                // Yucky unsafe here
                // But Globals needs to be available from callbacks and it's
                // really tricky to not use unsafe with UI/event loops
                let globals_copy = unsafe { &mut *(globals as *mut Globals) };
                let reg_for_handler = reg_for_init.clone();
                let r = web_view::builder()
                    .title(Box::leak(Box::new(title.str().to_owned())))
                    .content(web_view::Content::Html(content.str()))
                    .size(width, height)
                    .resizable(resizable)
                    .debug(debug)
                    .frameless(frameless)
                    .user_data(())
                    .invoke_handler(move |_webview, arg| {
                        if arg.starts_with("eval/") {
                            let mut iter = arg.splitn(4, '/');
                            iter.next().unwrap(); // eval
                            let id: usize = iter.next().unwrap().parse().unwrap();
                            let type_: usize = iter.next().unwrap().parse().unwrap();
                            let encoded_data = iter.next().unwrap();
                            let result = match type_ {
                                0 => {
                                    // Raw string
                                    Ok(Value::from(encoded_data))
                                }
                                1 => {
                                    // JSON
                                    let sval = serde_json::from_str(encoded_data)
                                        .expect("Bad JSON data from webview");
                                    Ok(super::json::from_serde(sval)
                                        .expect("Could not convert JSON data from webview to mtots"))
                                }
                                2 => {
                                    // Reference
                                    let wv = globals_copy
                                        .stash()
                                        .get::<WeakHandle<WV>>()
                                        .unwrap()
                                        .clone();
                                    let id: usize =
                                        encoded_data.parse().expect("Invalid JS reference ID");
                                    let jsref = JsRef { wv, id };
                                    Ok(globals_copy.new_handle(jsref).unwrap().into())
                                }
                                3 => {
                                    // Error
                                    Err(rterr!("JSError: {}", encoded_data))
                                }
                                _ => panic!("Invalid JS eval result type: {:?}", type_),
                            };
                            let resolve = reg_for_handler
                                .borrow_mut()
                                .resolve_map
                                .remove(&id)
                                .unwrap();
                            resolve(globals_copy, result);
                        } else {
                            let arg = Value::from(arg);
                            let r = handler.apply(globals_copy, vec![arg], None);
                            ordie(globals_copy, r);
                        }
                        Ok(())
                    })
                    .build();
                let wv = WV(mtry!(r));
                let wv_handle = globals.new_handle(wv)?;
                let weak_wv_handle = wv_handle.downgrade();
                globals.stash_mut().set::<WeakHandle<WV>>(weak_wv_handle)?;
                Ok(wv_handle.into())
            },
        );

        let reg_for_webview = reg.clone();
        m.class::<WV, _>("WebView", move |cls| {
            cls.ifunc("run", (), "", |owner, _globals, _, _| {
                // This is kinda yucky, but extracting the webview like this when doing
                // the step, allows me to return the 'WebView' value in init
                // Otherwise, any operation that requires borrow_mut() inside any callback
                // will cause a panic
                let webview = unsafe {
                    &mut *(&mut owner.borrow_mut().0 as *mut web_view::WebView<'static, ()>)
                };
                loop {
                    match webview.step() {
                        Some(Ok(_)) => {}
                        Some(Err(e)) => return Err(rterr!("WebView error: {:?}", e)),
                        None => return Ok(Value::Nil),
                    }
                }
            });
            cls.ifunc(
                "eval0",
                ["js"],
                "Evaluates a js code snippet; returns nil",
                |owner, _globals, args, _| {
                    let mut args = args.into_iter();
                    let js = args.next().unwrap().into_string()?;
                    owner.borrow_mut().eval0(js.str())?;
                    Ok(().into())
                },
            );

            let reg_for_evalstr = reg_for_webview.clone();
            cls.ifunc(
                "evals",
                ["js"],
                concat!(
                    "Evaluates a js code snippet, and (asynchronously) ",
                    "returns the result (as a string)",
                ),
                move |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let js = args.next().unwrap().into_string()?;
                    let id = reg_for_evalstr.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_evalstr
                            .borrow_mut()
                            .resolve_map
                            .insert(id, resolve);
                    });
                    owner.borrow_mut().evals(id, js.str())?;
                    Ok(promise.into())
                },
            );

            let reg_for_evaljson = reg_for_webview.clone();
            cls.ifunc(
                "evalj",
                ["js"],
                concat!(
                    "Evaluates a js code snippet, and (asynchronously) ",
                    "returns the result as a deserialized JSON value",
                ),
                move |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let js = args.next().unwrap().into_string()?;
                    let id = reg_for_evaljson.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_evaljson
                            .borrow_mut()
                            .resolve_map
                            .insert(id, resolve);
                    });
                    owner.borrow_mut().evalj(id, js.str())?;
                    Ok(promise.into())
                },
            );

            let reg_for_evalref = reg_for_webview.clone();
            cls.ifunc(
                "evalr",
                ["js"],
                concat!(
                    "Evaluates a js code snippet, and (asynchronously) ",
                    "returns the result as a JsRef",
                ),
                move |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let js = args.next().unwrap().into_string()?;
                    let id = reg_for_evalref.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_evalref
                            .borrow_mut()
                            .resolve_map
                            .insert(id, resolve);
                    });
                    owner.borrow_mut().evalr(id, js.str())?;
                    Ok(promise.into())
                },
            );

            cls.ifunc("set_title", ["title"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let title = args.next().unwrap().into_string()?;
                let r = owner.borrow_mut().0.set_title(title.str());
                mtry!(r);
                Ok(().into())
            });
            cls.ifunc(
                "set_fullscreen",
                ["title"],
                "",
                |owner, _globals, args, _| {
                    let mut args = args.into_iter();
                    let fullscreen = args.next().unwrap().truthy();
                    owner.borrow_mut().0.set_fullscreen(fullscreen);
                    Ok(().into())
                },
            );
            cls.ifunc("exit", (), "", |owner, _globals, _args, _| {
                owner.borrow_mut().0.exit();
                Ok(().into())
            });
        });

        let reg_for_jsref = reg.clone();
        m.class::<JsRef, _>("JsRef", move |cls| {
            cls.doc("Reference to a Javascript object");

            let reg_for_fref = reg_for_jsref.clone();
            cls.ifunc(
                "fref",
                ["name"],
                "Gets the field of an object, and returns the result as a JsRef",
                move |handle, globals, args, _| {
                    let mut args = args.into_iter();
                    let name = args.next().unwrap().into_string()?;
                    let ref_ = handle.borrow();
                    let wv = ref_.wv()?;
                    let req_id = reg_for_fref.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_fref
                            .borrow_mut()
                            .resolve_map
                            .insert(req_id, resolve);
                    });
                    wv.borrow_mut()
                        .evalr(req_id, &format!("$$REFS.a{}.{}", ref_.id, name))?;
                    Ok(promise.into())
                },
            );

            let reg_for_f = reg_for_jsref.clone();
            cls.ifunc(
                "f",
                ["name"],
                "Gets the field of the object, and returns the result as deserialized JSON",
                move |handle, globals, args, _| {
                    let mut args = args.into_iter();
                    let name = args.next().unwrap().into_string()?;
                    let ref_ = handle.borrow();
                    let wv = ref_.wv()?;
                    let req_id = reg_for_f.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_f
                            .borrow_mut()
                            .resolve_map
                            .insert(req_id, resolve);
                    });
                    wv.borrow_mut()
                        .evalj(req_id, &format!("$$REFS.a{}.{}", ref_.id, name))?;
                    Ok(promise.into())
                },
            );

            cls.ifunc(
                "sf",
                ["name", "value"],
                "Sets the field of the object",
                move |handle, globals, args, _| {
                    let mut args = args.into_iter();
                    let name = args.next().unwrap().into_string()?;
                    let value = arg_for_js(globals, args.next().unwrap())?;
                    let ref_ = handle.borrow();
                    let wv = ref_.wv()?;
                    wv.borrow_mut()
                        .eval0(&format!("$$REFS.a{}.{} = {}", ref_.id, name, value))?;
                    Ok(().into())
                },
            );

            let reg_for_mr = reg_for_jsref.clone();
            cls.ifunc(
                "mr",
                ArgSpec::builder().req("name").var("args"),
                "Calls a method, and returns the result as a JsRef",
                move |handle, globals, args, _| {
                    let mut args = args.into_iter();
                    let name = args.next().unwrap().into_string()?;
                    let args = args_for_js(globals, args)?;
                    let ref_ = handle.borrow();
                    let wv = ref_.wv()?;
                    let req_id = reg_for_mr.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_mr
                            .borrow_mut()
                            .resolve_map
                            .insert(req_id, resolve);
                    });
                    wv.borrow_mut()
                        .evalr(req_id, &format!("$$REFS.a{}.{}{}", ref_.id, name, args))?;
                    Ok(promise.into())
                },
            );

            let reg_for_m = reg_for_jsref.clone();
            cls.ifunc(
                "m",
                ArgSpec::builder().req("name").var("args"),
                "Calls a method, and returns the result as deserialized JSON",
                move |handle, globals, args, _| {
                    let mut args = args.into_iter();
                    let name = args.next().unwrap().into_string()?;
                    let args = args_for_js(globals, args)?;
                    let ref_ = handle.borrow();
                    let wv = ref_.wv()?;
                    let req_id = reg_for_m.borrow_mut().new_id();
                    let promise = Promise::new(globals, |_globals, resolve| {
                        reg_for_m
                            .borrow_mut()
                            .resolve_map
                            .insert(req_id, resolve);
                    });
                    wv.borrow_mut()
                        .evalj(req_id, &format!("$$REFS.a{}.{}{}", ref_.id, name, args))?;
                    Ok(promise.into())
                },
            );
        });
    })
}

/// wrapper around WebView, so that they can be used in mtots
struct WV(web_view::WebView<'static, ()>);

impl WV {
    fn eval0(&mut self, js: &str) -> Result<()> {
        mtry!(self.0.eval(js));
        Ok(())
    }
    fn evaltry(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.eval0(&format!("try{{{}}}catch(e){{$$ERR({},e)}}", js, req_id))
    }
    fn evals(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.evaltry(req_id, &format!("external.invoke('eval/{}/0/'+({}))", req_id, js))
    }
    fn evalj(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.evaltry(req_id, &format!("$$RETJSON({},{})", req_id, js))
    }
    fn evalr(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.evaltry(req_id, &format!("$$RETREF({},{})", req_id, js))
    }
}

#[derive(Default)]
struct JsRequestRegistry {
    next_id: usize,
    resolve_map: HashMap<usize, Box<dyn FnOnce(&mut Globals, Result<Value>)>>,
}
impl JsRequestRegistry {
    pub fn new_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

fn args_for_js<I: IntoIterator<Item = Value>>(globals: &mut Globals, args: I) -> Result<String> {
    let args = args
        .into_iter()
        .map(|arg| arg_for_js(globals, arg))
        .collect::<Result<Vec<String>>>()?;
    Ok(format!("({})", args.join(",")))
}

fn arg_for_js(globals: &mut Globals, value: Value) -> Result<String> {
    match value {
        Value::Nil | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            let sval = super::json::to_serde(value)?;
            Ok(mtry!(serde_json::to_string(&sval)))
        }
        Value::List(list) => {
            let mut parts = Vec::new();
            for x in list.borrow().iter() {
                parts.push(arg_for_js(globals, x.clone())?);
            }
            Ok(format!("[{}]", parts.join(",")))
        }
        Value::Map(map) => {
            let mut parts = Vec::new();
            for (key, val) in map.borrow().iter() {
                parts.push(format!(
                    "{}:{}",
                    Value::from(key).into_string()?,
                    arg_for_js(globals, val.clone())?,
                ));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
        value if value.is_handle::<JsRef>() => {
            let handle = value.into_handle::<JsRef>()?;
            Ok(format!("$$REFS.a{}", handle.borrow().id))
        }
        value => Err(rterr!("{:?} could not be converted to a JS value", value)),
    }
}
