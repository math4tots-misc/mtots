use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Stashable;
use crate::Symbol;
use crate::Value;
use crate::HCow;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::Font;
use ggez::graphics::TextFragment;
use ggez::graphics::Text;
use ggez::graphics::DrawParam;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod conv;

use conv::*;

pub const NAME: &str = "a._ggez";

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

    keycode_map: HashMap<ggez::event::KeyCode, Symbol>,
    mouse_button_map: HashMap<ggez::event::MouseButton, Symbol>,
}

impl EventHandler {
    fn translate_keycode(&mut self, keycode: ggez::event::KeyCode) -> Symbol {
        match self.keycode_map.entry(keycode) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                *entry.insert(Symbol::from(format!("{:?}", keycode)))
            }
            std::collections::hash_map::Entry::Occupied(entry) => *entry.get(),
        }
    }
    fn translate_button(&mut self, btn: ggez::event::MouseButton) -> Value {
        if let ggez::event::MouseButton::Other(x) = btn {
            Value::Int(x as i64)
        } else {
            match self.mouse_button_map.entry(btn) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    (*entry.insert(Symbol::from(format!("{:?}", btn)))).into()
                }
                std::collections::hash_map::Entry::Occupied(entry) => (*entry.get()).into(),
            }
        }
    }
}

impl ggez::event::EventHandler for EventHandler {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(update) = &self.update {
            let r = Eval::call(&mut self.globals, update, vec![]);
            ordie(&mut self.globals, r);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(draw) = &self.draw {
            let r = Eval::call(&mut self.globals, draw, vec![]);
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
            let r = Eval::call(&mut self.globals, mouse_down, vec![x, y, btn]);
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
            let r = Eval::call(&mut self.globals, mouse_up, vec![x, y, btn]);
            ordie(&mut self.globals, r);
        }
    }
    fn mouse_motion_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32, dx: f32, dy: f32) {
        let x = (x as f64).into();
        let y = (y as f64).into();
        let dx = (dx as f64).into();
        let dy = (dy as f64).into();
        if let Some(mouse_move) = &self.mouse_move {
            let r = Eval::call(&mut self.globals, mouse_move, vec![x, y, dx, dy]);
            ordie(&mut self.globals, r);
        }
    }
    fn mouse_wheel_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32) {
        let x = (x as f64).into();
        let y = (y as f64).into();
        if let Some(mouse_wheel) = &self.mouse_wheel {
            let r = Eval::call(&mut self.globals, mouse_wheel, vec![x, y]);
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
            let r = Eval::call(&mut self.globals, key_down, vec![key.into(), repeat.into()]);
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
            let r = Eval::call(&mut self.globals, &key_up, vec![key.into()]);
            ordie(&mut self.globals, r);
        }
    }
    fn text_input_event(&mut self, _ctx: &mut ggez::Context, ch: char) {
        if let Some(text_input) = self.text_input.clone() {
            let ch = self.globals.char_to_val(ch);
            let r = Eval::call(&mut self.globals, &text_input, vec![ch]);
            ordie(&mut self.globals, r);
        }
    }
    fn resize_event(&mut self, _ctx: &mut ggez::Context, width: f32, height: f32) {
        let width = (width as f64).into();
        let height = (height as f64).into();
        if let Some(resize) = self.resize.clone() {
            let r = Eval::call(&mut self.globals, &resize, vec![width, height]);
            ordie(&mut self.globals, r);
        }
    }
}

struct Stash {
    ctx: &'static mut ggez::Context,
}

impl Stashable for Stash {}

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let mut map = HashMap::<RcStr, Value>::new();

    let textcls = globals.new_class0("a._ggez::Text", vec![], vec![])?;
    globals.set_handle_class::<Text>(textcls)?;

    map.extend(
        vec![
            NativeFunction::snew(
                "run",
                (
                    &[
                        "name",
                        "author",
                        "init",
                        "update",
                        "draw",
                        "mouse_down",
                        "mouse_up",
                        "mouse_move",
                        "mouse_wheel",
                        "key_down",
                        "key_up",
                        "text_input",
                        "resize",
                    ],
                    &[],
                    None,
                    None,
                ),
                |globals: &mut Globals, args, _| {
                    let mut args = args.into_iter();
                    let name = Eval::expect_string(globals, &args.next().unwrap())?.clone();
                    let author = Eval::expect_string(globals, &args.next().unwrap())?.clone();
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
                    globals.escape_to_trampoline(move |mut globals| {
                        let (mut ctx, mut event_loop) =
                            ggez::ContextBuilder::new(name.str(), author.str())
                                .build()
                                .unwrap();
                        let stash = Stash {
                            // kinda yucky to use unsafe here, but it would be quite a bit of work to avoid this
                            ctx: unsafe { std::mem::transmute::<&mut ggez::Context, _>(&mut ctx) },
                        };
                        let r = globals.set_stash(stash);
                        ordie(&mut globals, r);

                        if let Some(init) = init {
                            let r = Eval::call(&mut globals, &init, vec![]);
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
                        event_handler.globals.delete_stash::<Stash>();
                    })
                },
            ),
            NativeFunction::snew(
                "clear",
                (&["color"], &[], None, None),
                |globals, args, _| {
                    let ctx = getctx(globals)?;
                    let color = to_color(globals, &args[0])?;
                    ggez::graphics::clear(ctx, color);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::snew(
                "print",
                (&["text", "x", "y"], &[], None, None),
                |globals, args, _| {
                    let mut args = args.into_iter();
                    let text = as_text(globals, args.next().unwrap())?;
                    let x = Eval::expect_floatlike(globals, &args.next().unwrap())? as f32;
                    let y = Eval::expect_floatlike(globals, &args.next().unwrap())? as f32;
                    let ctx = getctx(globals)?;
                    let r = text.with(|text| ggez::graphics::draw(ctx, text, DrawParam::default().dest([x, y])));
                    conve(globals, r)?;
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::snew(
                "new_text",
                (&["text"], &[], None, None),
                |globals, args, _| {
                    let mut args = args.into_iter();
                    let text = as_text(globals, args.next().unwrap())?;
                    Ok(Eval::into_handle(globals, text)?.into())
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

fn getornil(value: Value) -> Option<Value> {
    match value {
        Value::Nil => None,
        value => Some(value),
    }
}

fn ordie<R>(globals: &mut Globals, r: EvalResult<R>) -> R {
    match r {
        Ok(r) => r,
        Err(_) => {
            assert!(globals.print_if_error());
            std::process::exit(1)
        }
    }
}

fn getstash(globals: &mut Globals) -> EvalResult<Rc<RefCell<Stash>>> {
    globals
        .get_from_stash_or_else(|globals| globals.set_exc_str(&format!("ggez not yet initialized")))
}

fn getctx(globals: &mut Globals) -> EvalResult<&'static mut ggez::Context> {
    // also yucky unsafe here, but kind of follows from the whole situation
    Ok(unsafe {
        std::mem::transmute::<&mut ggez::Context, _>(&mut getstash(globals)?.borrow_mut().ctx)
    })
}

fn conve<E: std::error::Error, R>(globals: &mut Globals, e: Result<R, E>) -> EvalResult<R> {
    match e {
        Ok(r) => Ok(r),
        Err(e) => globals.set_exc_str(&format!("{:?}", e)),
    }
}
