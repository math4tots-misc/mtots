//! JSON bindings
use crate::mtry;
use crate::rterr;
use crate::ordie;
use crate::ArgSpec;
use crate::Globals;
use crate::NativeModule;
use crate::Promise;
use crate::Result;
use crate::Value;
use std::collections::HashMap;
use std::convert::TryFrom;

pub const NAME: &str = "a.webview";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.func(
            "init",
            ArgSpec::builder()
                .def("title", "some-window")
                .def("content", "<html></html>")
                .def("size", [320, 480])
                .def("resizeable", false)
                .def("debug", false)
                .def("handler", ()),
            "",
            |globals, args, _| {
                let mut args = args.into_iter();
                let title = args.next().unwrap().into_string()?;
                let content = args.next().unwrap().into_string()?;
                let [width, height] = <[i32; 2]>::try_from(args.next().unwrap())?;
                let resizable = args.next().unwrap().truthy();
                let debug = args.next().unwrap().truthy();
                let handler = args.next().unwrap();

                // Yucky unsafe here
                // But Globals needs to be available from callbacks and it's
                // really tricky to not use unsafe with UI/event loops
                let globals_copy = unsafe { &mut *(globals as *mut Globals) };
                let r = web_view::builder()
                    .title(Box::leak(Box::new(title.str().to_owned())))
                    .content(web_view::Content::Html(content.str()))
                    .size(width, height)
                    .resizable(resizable)
                    .debug(debug)
                    .user_data(())
                    .invoke_handler(move |_webview, arg| {
                        if arg.starts_with("eval/") {
                            let mut iter = arg.splitn(3, '/');
                            iter.next().unwrap(); // eval
                            let id: usize = iter.next().unwrap().parse().unwrap();
                            let resolve = {
                                let mut reg =
                                    globals_copy.stash_mut().get_mut::<JsRequestRegistry>().unwrap();
                                reg.resolve_map.remove(&id).unwrap()
                            };
                            resolve(globals_copy, Ok(Value::from(iter.next().unwrap())));
                        } else {
                            let arg = Value::from(arg);
                            let r = handler.apply(globals_copy, vec![arg], None);
                            ordie(globals_copy, r);
                        }
                        Ok(())
                    })
                    .build();
                let wv = WV(mtry!(r));
                Ok(globals.new_handle(wv)?.into())
            },
        );
        m.class::<WV, _>("WebView", |cls| {
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
            cls.ifunc("eval", ["js"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let js = args.next().unwrap().into_string()?;
                let r = owner.borrow_mut().0.eval(js.str());
                mtry!(r);
                Ok(().into())
            });
            cls.ifunc(
                "async_eval",
                ["js"],
                concat!(
                    "Evaluates a js code snippet, and (asynchronously) ",
                    "returns the result (as a string)",
                ),
                |owner, globals, args, _| {
                    let mut args = args.into_iter();
                    let js = args.next().unwrap().into_string()?;
                    let id = {
                        let mut reg = globals.stash_mut().get_mut::<JsRequestRegistry>()?;
                        reg.next_id += 1;
                        reg.next_id
                    };
                    let promise = Promise::new(globals, |globals, resolve| {
                        let mut reg = globals.stash_mut().get_mut::<JsRequestRegistry>().unwrap();
                        reg.resolve_map.insert(id, resolve);
                    });
                    let r = owner.borrow_mut().0.eval(&format!(
                        "external.invoke('eval/{}/' + {})",
                        id,
                        js.str(),
                    ));
                    mtry!(r);
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
            cls.ifunc(
                "exit",
                (),
                "",
                |owner, _globals, _args, _| {
                    owner.borrow_mut().0.exit();
                    Ok(().into())
                },
            );
        });
        m.action(|globals, _map| {
            globals.stash_mut().set(JsRequestRegistry::default())?;
            Ok(())
        });
    })
}

/// wrapper around WebView, so that they can be used in mtots
struct WV(web_view::WebView<'static, ()>);

#[derive(Default)]
struct JsRequestRegistry {
    next_id: usize,
    resolve_map: HashMap<usize, Box<dyn FnOnce(&mut Globals, Result<Value>)>>,
}
