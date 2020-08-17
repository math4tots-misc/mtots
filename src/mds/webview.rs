//! JSON bindings
use crate::mtry;
use crate::ordie;
use crate::ArgSpec;
use crate::NativeModule;
use crate::Value;
use std::convert::TryFrom;

pub const NAME: &str = "a.webview";

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.func(
            "run",
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
                let r = web_view::builder()
                    .title(title.str())
                    .content(web_view::Content::Html(content.str()))
                    .size(width, height)
                    .resizable(resizable)
                    .debug(debug)
                    .user_data(())
                    .invoke_handler(|webview, arg| {
                        let webview = WV(unsafe { std::mem::transmute(webview) });
                        let r = globals.new_handle::<WV>(webview);
                        let webview = Value::from(ordie(globals, r));
                        let arg = Value::from(arg);
                        let r = handler.apply(globals, vec![webview, arg], None);
                        ordie(globals, r);
                        Ok(())
                    })
                    .run();
                mtry!(r);
                Ok(Value::Nil)
            },
        );
        m.class::<WV, _>("WebView", |cls| {
            cls.ifunc("eval", ["js"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let js = args.next().unwrap().into_string()?;
                let r = owner.borrow_mut().0.eval(js.str());
                mtry!(r);
                Ok(().into())
            });
            cls.ifunc("set_title", ["title"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let title = args.next().unwrap().into_string()?;
                let r = owner.borrow_mut().0.set_title(title.str());
                mtry!(r);
                Ok(().into())
            });
            cls.ifunc("set_fullscreen", ["title"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let fullscreen = args.next().unwrap().truthy();
                owner.borrow_mut().0.set_fullscreen(fullscreen);
                Ok(().into())
            });
        });
    })
}

/// wrapper around WebView, so that they can be used in mtots
struct WV(&'static mut web_view::WebView<'static, ()>);
