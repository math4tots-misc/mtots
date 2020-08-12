use crate::mtry;
use crate::ordie;
use crate::rterr;
use crate::ArgSpec;
use crate::ConvertValue;
use crate::Error;
use crate::Globals;
use crate::NativeModule;
use crate::RcStr;
use crate::Result;
use crate::Value;
use ggez::graphics::DrawParam;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

pub mod graphics;

pub(super) fn add(globals: &mut Globals) {
    globals.add(new()).unwrap();
    globals.add(graphics::new()).unwrap();
}

pub const NAME: &str = "a.ggez";

struct EventHandler {
    globals: Globals,
    update: Option<Value>,
    draw: Option<Value>,
    mouse_down: Option<Value>,
    mouse_up: Option<Value>,
    mouse_move: Option<Value>,
    mouse_wheel: Option<Value>,
    key_down: Option<Value>,
    key_up: Option<Value>,
    text_input: Option<Value>,
    resize: Option<Value>,

    keycode_map: HashMap<ggez::event::KeyCode, RcStr>,
    mouse_button_map: HashMap<ggez::event::MouseButton, RcStr>,
}

impl EventHandler {
    fn translate_keycode(&mut self, keycode: ggez::event::KeyCode) -> Value {
        match self.keycode_map.entry(keycode) {
            std::collections::hash_map::Entry::Vacant(entry) => (*entry
                .insert(RcStr::from(format!("{:?}", keycode))))
            .clone()
            .into(),
            std::collections::hash_map::Entry::Occupied(entry) => (*entry.get()).clone().into(),
        }
    }
    fn translate_button(&mut self, btn: ggez::event::MouseButton) -> Value {
        if let ggez::event::MouseButton::Other(x) = btn {
            Value::from(x)
        } else {
            match self.mouse_button_map.entry(btn) {
                std::collections::hash_map::Entry::Vacant(entry) => (*entry
                    .insert(RcStr::from(format!("{:?}", btn))))
                .clone()
                .into(),
                std::collections::hash_map::Entry::Occupied(entry) => (*entry.get()).clone().into(),
            }
        }
    }
}

impl ggez::event::EventHandler for EventHandler {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(update) = &self.update {
            let r = update.apply(&mut self.globals, vec![], None);
            ordie(&mut self.globals, r);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(draw) = &self.draw {
            let r = draw.apply(&mut self.globals, vec![], None);
            ordie(&mut self.globals, r);
            ggez::graphics::present(ctx)?;
        }
        std::thread::yield_now();
        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        btn: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let btn = self.translate_button(btn);
        let x = (x as f64).into();
        let y = (y as f64).into();
        if let Some(mouse_down) = &self.mouse_down {
            let r = mouse_down.apply(&mut self.globals, vec![x, y, btn], None);
            ordie(&mut self.globals, r);
        }
    }
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        btn: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let btn = self.translate_button(btn);
        let x = (x as f64).into();
        let y = (y as f64).into();
        if let Some(mouse_up) = &self.mouse_up {
            let r = mouse_up.apply(&mut self.globals, vec![x, y, btn], None);
            ordie(&mut self.globals, r);
        }
    }
    fn mouse_motion_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32, dx: f32, dy: f32) {
        let x = (x as f64).into();
        let y = (y as f64).into();
        let dx = (dx as f64).into();
        let dy = (dy as f64).into();
        if let Some(mouse_move) = &self.mouse_move {
            let r = mouse_move.apply(&mut self.globals, vec![x, y, dx, dy], None);
            ordie(&mut self.globals, r);
        }
    }
    fn mouse_wheel_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32) {
        let x = (x as f64).into();
        let y = (y as f64).into();
        if let Some(mouse_wheel) = &self.mouse_wheel {
            let r = mouse_wheel.apply(&mut self.globals, vec![x, y], None);
            ordie(&mut self.globals, r);
        }
    }
    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: ggez::event::KeyCode,
        _keymods: ggez::event::KeyMods,
        repeat: bool,
    ) {
        if keycode == ggez::event::KeyCode::Escape {
            ggez::event::quit(ctx);
            return;
        }
        let key = self.translate_keycode(keycode);
        if let Some(key_down) = &self.key_down {
            let r = key_down.apply(&mut self.globals, vec![key.into(), repeat.into()], None);
            ordie(&mut self.globals, r);
        }
    }
    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: ggez::event::KeyCode,
        _keymods: ggez::event::KeyMods,
    ) {
        let key = self.translate_keycode(keycode);
        if let Some(key_up) = self.key_up.clone() {
            let r = key_up.apply(&mut self.globals, vec![key.into()], None);
            ordie(&mut self.globals, r);
        }
    }
    fn text_input_event(&mut self, _ctx: &mut ggez::Context, ch: char) {
        if let Some(text_input) = self.text_input.clone() {
            let ch = Value::from(ch);
            let r = text_input.apply(&mut self.globals, vec![ch], None);
            ordie(&mut self.globals, r);
        }
    }
    fn resize_event(&mut self, _ctx: &mut ggez::Context, width: f32, height: f32) {
        let width = (width as f64).into();
        let height = (height as f64).into();
        if let Some(resize) = self.resize.clone() {
            let r = resize.apply(&mut self.globals, vec![width, height], None);
            ordie(&mut self.globals, r);
        }
    }
}

struct Stash {
    ctx: &'static mut ggez::Context,
}

pub(super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.func(
            "run",
            ArgSpec::builder()
                .def("name", "")
                .def("author", "")
                .def("init", ())
                .def("update", ())
                .def("draw", ())
                .def("mouse_down", ())
                .def("mouse_up", ())
                .def("mouse_move", ())
                .def("mouse_wheel", ())
                .def("key_down", ())
                .def("key_up", ())
                .def("text_input", ())
                .def("resize", ())
                .def("resource_paths", []),
            "",
            |globals, args, _| {
                let mut args = args.into_iter();
                let name = args.next().unwrap().into_string()?;
                let author = args.next().unwrap().into_string()?;
                let init = getornil(args.next().unwrap());
                let update = getornil(args.next().unwrap());
                let draw = getornil(args.next().unwrap());
                let mouse_down = getornil(args.next().unwrap());
                let mouse_up = getornil(args.next().unwrap());
                let mouse_move = getornil(args.next().unwrap());
                let mouse_wheel = getornil(args.next().unwrap());
                let key_down = getornil(args.next().unwrap());
                let key_up = getornil(args.next().unwrap());
                let text_input = getornil(args.next().unwrap());
                let resize = getornil(args.next().unwrap());
                let resource_paths = Vec::<RcStr>::try_from(args.next().unwrap())?;
                globals.request_trampoline(move |mut globals| {
                    let mut builder = ggez::ContextBuilder::new(name.str(), author.str());
                    for resource_path in resource_paths {
                        builder = builder.add_resource_path(PathBuf::from(resource_path.str()));
                    }
                    let (mut ctx, mut event_loop) = builder.build().unwrap();
                    let stash = Stash {
                        // kinda yucky to use unsafe here, but it would be quite a bit of work to avoid this
                        ctx: unsafe { std::mem::transmute::<&mut ggez::Context, _>(&mut ctx) },
                    };
                    let r = globals.stash_mut().set(stash);
                    ordie(&mut globals, r);

                    if let Some(init) = init {
                        let r = init.apply(&mut globals, vec![], None);
                        ordie(&mut globals, r);
                    }

                    let mut event_handler = EventHandler {
                        globals,
                        update,
                        draw,
                        mouse_down,
                        mouse_up,
                        mouse_move,
                        mouse_wheel,
                        key_down,
                        key_up,
                        text_input,
                        resize,
                        keycode_map: HashMap::new(),
                        mouse_button_map: HashMap::new(),
                    };

                    match ggez::event::run(&mut ctx, &mut event_loop, &mut event_handler) {
                        Ok(_) => {}
                        Err(e) => eprintln!("ggez error: {:?}", e),
                    }
                    event_handler.globals.stash_mut().remove::<Stash>();
                })
            },
        );
    })
}

fn getctx(globals: &mut Globals) -> Result<&'static mut ggez::Context> {
    use std::ops::DerefMut;
    // also yucky unsafe here, but kind of follows from the whole situation
    let stash = globals.stash_mut();
    if !stash.has::<Stash>() {
        return Err(rterr!("GGEZ context used before being initialized"));
    }
    let mut stash = stash.get_mut::<Stash>()?;
    let stash = stash.deref_mut();
    Ok(unsafe { std::mem::transmute::<&mut ggez::Context, _>(stash.ctx) })
}

fn getornil(value: Value) -> Option<Value> {
    match value {
        Value::Nil => None,
        value => Some(value),
    }
}
