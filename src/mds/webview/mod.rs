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
            function $$RETX(reqId, x) {
                switch (typeof x) {
                    case 'undefined':
                        return $$RETJSON(reqId, null);
                    case 'boolean':
                    case 'number':
                    case 'bigint':
                    case 'string':
                        return $$RETJSON(reqId, x);
                    case 'object':
                        return x === null ? $$RETJSON(reqId, x) : $$RETREF(reqId, x);
                    default:
                        return $$RETREF(reqId, x);
                }
            }
            function $$ERR(reqId, e) {
                external.invoke('eval/' + reqId + '/3/' + e);
            }
            function $$TIMEOUT(reqId) {
                external.invoke('eval/' + reqId + '/4/');
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
                                    Ok(super::json::from_serde(sval).expect(
                                        "Could not convert JSON data from webview to mtots",
                                    ))
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
                                4 => {
                                    // Timeout
                                    Ok(Value::Nil)
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

            let reg_for_evals = reg_for_webview.clone();
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
                    let promise = reg_for_evals.borrow_mut().evals(
                        globals,
                        &mut owner.borrow_mut(),
                        js.str(),
                    )?;
                    Ok(promise.into())
                },
            );

            let reg_for_evalj = reg_for_webview.clone();
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
                    let promise = reg_for_evalj.borrow_mut().evalj(
                        globals,
                        &mut owner.borrow_mut(),
                        js.str(),
                    )?;
                    Ok(promise.into())
                },
            );

            let reg_for_evalr = reg_for_webview.clone();
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
                    let promise = reg_for_evalr.borrow_mut().evalr(
                        globals,
                        &mut owner.borrow_mut(),
                        js.str(),
                    )?;
                    Ok(promise.into())
                },
            );

            let reg_for_evalx = reg_for_webview.clone();
            cls.ifunc(
                "evalx",
                ["js"],
                concat!(
                    "Evaluates a js code snippet, and (asynchronously) ",
                    "returns the result as a value (if it is a primitive type) or ",
                    "as a JsRef otherwise",
                ),
                move |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let js = args.next().unwrap().into_string()?;
                    let promise = reg_for_evalx.borrow_mut().evalx(
                        globals,
                        &mut owner.borrow_mut(),
                        js.str(),
                    )?;
                    Ok(promise.into())
                },
            );

            let reg_for_timeout = reg_for_webview.clone();
            cls.ifunc("timeout", ["nsec"], "", move |owner, globals, args, _| {
                let mut args = args.into_iter();
                let nsec = args.next().unwrap().f64()?;
                let promise = reg_for_timeout.borrow_mut().timeout(
                    globals,
                    &mut owner.borrow_mut(),
                    (nsec * 1000.0) as usize,
                )?;
                Ok(promise.into())
            });

            let reg_for_bytes = reg_for_webview.clone();
            cls.ifunc(
                "blob",
                ArgSpec::builder().req("bytes").def("type", ""),
                concat!("Creates and (asynchronously) returns a JS reference to a Blob",),
                move |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let bytes = args.next().unwrap().convert::<Vec<u8>>(globals)?;
                    let bytes = bytes
                        .into_iter()
                        .map(|b| format!("{}", b))
                        .collect::<Vec<String>>();
                    let bstr = bytes.join(",");
                    let type_ = args.next().unwrap().into_string()?;
                    let promise = reg_for_bytes.borrow_mut().evalr(
                        globals,
                        &mut owner.borrow_mut(),
                        &format!("new Blob([{}],'{}')", bstr, type_),
                    )?;
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

            let reg_for_json = reg_for_jsref.clone();
            cls.sfunc(
                "json",
                ["ref"],
                concat!(
                    "Converts and (asynchronously) returns the JS reference ",
                    "as a JSON blob",
                ),
                move |globals, args, _| {
                    let mut args = args.into_iter();
                    let ref_ = args.next().unwrap().into_handle::<JsRef>()?;
                    let ref_ = ref_.borrow();
                    let wv = ref_.wv()?;
                    let mut wv = wv.borrow_mut();
                    let promise = reg_for_json.borrow_mut().evalj(
                        globals,
                        &mut wv,
                        &format!("$$REFS.a{}", ref_.id),
                    )?;
                    Ok(promise.into())
                },
            );

            cls.sfunc(
                "name",
                ["ref"],
                concat!(
                    "Given a JsRef, returns a string that when evaluated ",
                    "on the JS side will retrieve the reference",
                ),
                move |_globals, args, _| {
                    let mut args = args.into_iter();
                    let ref_ = args.next().unwrap().into_handle::<JsRef>()?;
                    let ref_ = ref_.borrow();
                    Ok(format!("$$REFS.a{}", ref_.id).into())
                },
            );

            let reg_for_getattr = reg_for_jsref.clone();
            cls.getattr(move |globals, handle, attrname| {
                let ref_ = handle.borrow();
                let wv = ref_.wv()?;
                let mut wv = wv.borrow_mut();
                let promise = reg_for_getattr.borrow_mut().evalx(
                    globals,
                    &mut wv,
                    &format!("$$REFS.a{}.{}", ref_.id, attrname),
                )?;
                Ok(Some(promise.into()))
            });

            cls.setattr(move |globals, handle, attrname, value| {
                let ref_ = handle.borrow();
                let wv = ref_.wv()?;
                let value = arg_for_js(globals, value)?;
                wv.borrow_mut()
                    .eval0(&format!("$$REFS.a{}.{}={}", ref_.id, attrname, value))?;
                Ok(())
            });

            let reg_for_method_call = reg_for_jsref.clone();
            cls.method_call(move |globals, handle, methodname, args, kwargs| {
                if kwargs.is_some() {
                    return Err(rterr!(
                        "kwargs are not allowed in method calls to Javascript objects"
                    ));
                }
                let args = args_for_js(globals, args)?;
                let ref_ = handle.borrow();
                let wv = ref_.wv()?;
                let mut wv = wv.borrow_mut();
                let promise = reg_for_method_call.borrow_mut().evalx(
                    globals,
                    &mut wv,
                    &format!("$$REFS.a{}.{}{}", ref_.id, methodname, args),
                )?;
                Ok(promise.into())
            });
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
        self.evaltry(
            req_id,
            &format!("external.invoke('eval/{}/0/'+({}))", req_id, js),
        )
    }
    fn evalj(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.evaltry(req_id, &format!("$$RETJSON({},{})", req_id, js))
    }
    fn evalr(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.evaltry(req_id, &format!("$$RETREF({},{})", req_id, js))
    }
    fn evalx(&mut self, req_id: usize, js: &str) -> Result<()> {
        self.evaltry(req_id, &format!("$$RETX({},{})", req_id, js))
    }
    fn timeout(&mut self, req_id: usize, millis: usize) -> Result<()> {
        self.evaltry(
            req_id,
            &format!("setTimeout($$TIMEOUT,{},{})", millis, req_id),
        )
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
    pub fn evals(
        &mut self,
        globals: &mut Globals,
        wv: &mut WV,
        js: &str,
    ) -> Result<Rc<RefCell<Promise>>> {
        let req_id = self.new_id();
        let promise = Promise::new(globals, |_globals, resolve| {
            self.resolve_map.insert(req_id, resolve);
        });
        wv.evals(req_id, js)?;
        Ok(promise)
    }
    pub fn evalx(
        &mut self,
        globals: &mut Globals,
        wv: &mut WV,
        js: &str,
    ) -> Result<Rc<RefCell<Promise>>> {
        let req_id = self.new_id();
        let promise = Promise::new(globals, |_globals, resolve| {
            self.resolve_map.insert(req_id, resolve);
        });
        wv.evalx(req_id, js)?;
        Ok(promise)
    }
    pub fn evalj(
        &mut self,
        globals: &mut Globals,
        wv: &mut WV,
        js: &str,
    ) -> Result<Rc<RefCell<Promise>>> {
        let req_id = self.new_id();
        let promise = Promise::new(globals, |_globals, resolve| {
            self.resolve_map.insert(req_id, resolve);
        });
        wv.evalj(req_id, js)?;
        Ok(promise)
    }
    pub fn evalr(
        &mut self,
        globals: &mut Globals,
        wv: &mut WV,
        js: &str,
    ) -> Result<Rc<RefCell<Promise>>> {
        let req_id = self.new_id();
        let promise = Promise::new(globals, |_globals, resolve| {
            self.resolve_map.insert(req_id, resolve);
        });
        wv.evalr(req_id, js)?;
        Ok(promise)
    }
    pub fn timeout(
        &mut self,
        globals: &mut Globals,
        wv: &mut WV,
        millis: usize,
    ) -> Result<Rc<RefCell<Promise>>> {
        let req_id = self.new_id();
        let promise = Promise::new(globals, |_globals, resolve| {
            self.resolve_map.insert(req_id, resolve);
        });
        wv.timeout(req_id, millis)?;
        Ok(promise)
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
