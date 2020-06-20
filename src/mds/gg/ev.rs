//! Functions for dealing with event handling
use super::try_;
use super::with_wctx;
use super::Shared;
use crate::Class;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Symbol;
use crate::Value;
use ggez::event;
use ggez::event::Axis;
use ggez::event::Button;
use ggez::event::EventHandler;
use ggez::event::GamepadId;
use ggez::event::KeyCode;
use ggez::event::KeyMods;
use ggez::event::MouseButton;
use ggez::graphics;
use ggez::Context;
use ggez::ContextBuilder;
use ggez::GameResult;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a.gg._nev";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![NativeFunction::sdnew0(
            sr,
            "start",
            &[
                "context_class",
                "name",
                "author",
                "sleep_per_frame",
                "update",
                "draw",
                "mouse_down",
                "mouse_up",
                "mouse_motion",
                "mouse_wheel",
                "key_down",
                "key_up",
                "text_input",
                "gamepad_button_down",
                "gamepad_axis",
                "resize",
            ],
            Some(concat!(
                "Starts the event loop\n",
                "  context_class: the script-land Context class to wrap opaque contexts\n",
                "    in before passing it to the callbacks\n",
                "  name: I'm not sure how it's used; it's just passed to ggez\n",
                "  author: just passed to ggez as is\n",
                "  sleep_per_frame: how many seconds (in float) to sleep between\n",
                "    each frame. May pass nil to use ggez::timer::yield_now instead.\n",
                "  Remaining parameters are all callbacks\n",
                "  All callbacks should be wrapped in Cells to allow\n",
                "  future customization. The Cells may contain nil, in which case\n",
                "  the callback will do nothing\n",
            )),
            |globals, args, _kwargs| {
                let mut args = args.into_iter();
                let context_class = Eval::expect_class(globals, &args.next().unwrap())?.clone();
                let name = Eval::expect_string(globals, &args.next().unwrap())?.clone();
                let author = Eval::expect_string(globals, &args.next().unwrap())?.clone();
                let sleep_per_frame = {
                    let sleep_per_frame_val = args.next().unwrap();
                    if let Value::Nil = sleep_per_frame_val {
                        None
                    } else {
                        Some(std::time::Duration::from_secs_f64(Eval::expect_floatlike(
                            globals,
                            &sleep_per_frame_val,
                        )?))
                    }
                };
                let update = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let draw = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let mouse_down = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let mouse_up = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let mouse_motion = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let mouse_wheel = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let key_down = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let key_up = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let text_input = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let gamepad_button_down =
                    Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let gamepad_axis = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                let resize = Eval::expect_cell(globals, &args.next().unwrap())?.clone();

                let shared: Rc<RefCell<Shared>> = globals.get_from_stash();

                // In this case, using the trampoline isn't strictly necessary, since
                // ggez is nice enough to allow non-static EventHandler to be passed
                // to event::run. however, not all game libraries are this polite.
                globals.escape_to_trampoline(move |mut globals| {
                    let (mut ctx, mut event_loop) = try_(
                        &mut globals,
                        ContextBuilder::new(name.str(), author.str()).build(),
                    )?;
                    let symbol_left = globals.intern_str("left");
                    let symbol_right = globals.intern_str("right");
                    let symbol_middle = globals.intern_str("middle");
                    let keycode_symbols = list_keycode_symbols(&mut globals);
                    let symbol_shift = globals.intern_str("shift");
                    let symbol_ctrl = globals.intern_str("ctrl");
                    let symbol_alt = globals.intern_str("alt");
                    let symbol_logo = globals.intern_str("logo");
                    let symbol_repeat = globals.intern_str("repeat");
                    let gamepad_button_map = list_gamepad_buttons(&mut globals);
                    let gamepad_axis_map = list_gamepad_axes(&mut globals);
                    let mut state = State {
                        shared,
                        globals,
                        context_class,
                        sleep_per_frame,
                        update,
                        draw,
                        mouse_down,
                        mouse_up,
                        mouse_motion,
                        mouse_wheel,
                        key_down,
                        key_up,
                        text_input,
                        gamepad_button_down,
                        gamepad_axis,
                        resize,
                        symbol_left,
                        symbol_right,
                        symbol_middle,
                        keycode_symbols,
                        symbol_shift,
                        symbol_ctrl,
                        symbol_alt,
                        symbol_logo,
                        symbol_repeat,
                        gamepad_button_map,
                        gamepad_axis_map,
                    };
                    match event::run(&mut ctx, &mut event_loop, &mut state) {
                        Ok(_) => {}
                        Err(e) => {
                            state.globals.print_if_error();
                            panic!("{}", e)
                        }
                    }
                    Ok(())
                })
            },
        )]
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

struct State {
    shared: Rc<RefCell<Shared>>,
    globals: Globals,
    context_class: Rc<Class>,
    sleep_per_frame: Option<std::time::Duration>,

    // event handlers
    update: Rc<RefCell<Value>>,
    draw: Rc<RefCell<Value>>,
    mouse_down: Rc<RefCell<Value>>,
    mouse_up: Rc<RefCell<Value>>,
    mouse_motion: Rc<RefCell<Value>>,
    mouse_wheel: Rc<RefCell<Value>>,
    key_down: Rc<RefCell<Value>>,
    key_up: Rc<RefCell<Value>>,
    text_input: Rc<RefCell<Value>>,
    gamepad_button_down: Rc<RefCell<Value>>,
    gamepad_axis: Rc<RefCell<Value>>,
    resize: Rc<RefCell<Value>>,

    // for mouse events
    symbol_left: Symbol,
    symbol_right: Symbol,
    symbol_middle: Symbol,

    // for keyboard events
    keycode_symbols: Vec<Symbol>,
    symbol_shift: Symbol,
    symbol_ctrl: Symbol,
    symbol_alt: Symbol,
    symbol_logo: Symbol,
    symbol_repeat: Symbol,

    // for gamepad events
    gamepad_button_map: HashMap<Button, Symbol>,
    gamepad_axis_map: HashMap<Axis, Symbol>,
}

impl State {
    fn call_handler(
        &mut self,
        name: &str,
        handler: Value,
        ctx: &mut Context,
        mut args: Vec<Value>,
    ) {
        let globals = &mut self.globals;
        let context_class = &self.context_class;
        let result = with_wctx(globals, ctx, |globals, opaque_ctx| {
            let ctx = Class::instantiate(context_class, globals, vec![opaque_ctx], None)?;
            let args = if args.is_empty() {
                vec![ctx]
            } else {
                args.insert(0, ctx);
                args
            };
            Eval::call(globals, &handler, args)
        });

        // I could return an EventLoop GameError here,
        // but not all EventHandler methods expect a GameResult
        if let Err(_) = result {
            assert!(globals.print_if_error());
            panic!("Uncaught exception while running handling {} event", name,);
        }
    }
    fn translate_mouse_button(&self, button: MouseButton) -> Value {
        match button {
            MouseButton::Left => Value::Symbol(self.symbol_left),
            MouseButton::Right => Value::Symbol(self.symbol_right),
            MouseButton::Middle => Value::Symbol(self.symbol_middle),
            MouseButton::Other(i) => Value::Int(i as i64),
        }
    }

    fn translate_keycode(&self, keycode: KeyCode) -> Value {
        // NOTE: we may panic here if list_keycodes() is not exhaustive
        Value::Symbol(self.keycode_symbols[keycode as usize])
    }

    fn translate_keymods(&self, keymods: KeyMods, repeat: bool) -> Vec<Value> {
        let mut ret = Vec::new();
        if keymods.contains(KeyMods::SHIFT) {
            ret.push(Value::Symbol(self.symbol_shift));
        }
        if keymods.contains(KeyMods::CTRL) {
            ret.push(Value::Symbol(self.symbol_ctrl));
        }
        if keymods.contains(KeyMods::ALT) {
            ret.push(Value::Symbol(self.symbol_alt));
        }
        if keymods.contains(KeyMods::LOGO) {
            ret.push(Value::Symbol(self.symbol_logo));
        }
        if repeat {
            ret.push(Value::Symbol(self.symbol_repeat));
        }
        ret
    }

    fn translate_gamepad_button(&self, button: Button) -> Value {
        // NOTE: we may panic here if list_gamepad_buttons is not exhaustive
        Value::Symbol(self.gamepad_button_map.get(&button).cloned().unwrap())
    }

    fn translate_axis(&self, axis: Axis) -> Value {
        // NOTE: we may panic here if list_gamepad_axes is not exhaustive
        Value::Symbol(self.gamepad_axis_map.get(&axis).cloned().unwrap())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let handler = self.update.borrow().clone();
        if let Value::Nil = &handler {
            return Ok(());
        }
        self.call_handler("update", handler, ctx, vec![]);
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let handler = self.draw.borrow().clone();
        if !handler.is_nil() {
            self.call_handler("draw", handler, ctx, vec![]);
        }
        graphics::present(ctx)?;
        if let Some(dur) = self.sleep_per_frame {
            std::thread::sleep(dur);
        } else {
            ggez::timer::yield_now();
        }
        Ok(())
    }
    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let handler = self.mouse_down.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let button = self.translate_mouse_button(button);
        let x = Value::Float(x as f64);
        let y = Value::Float(y as f64);
        self.call_handler("mouse_down", handler, ctx, vec![button, x, y]);
    }
    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let handler = self.mouse_up.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let button = self.translate_mouse_button(button);
        let x = Value::Float(x as f64);
        let y = Value::Float(y as f64);
        self.call_handler("mouse_up", handler, ctx, vec![button, x, y]);
    }
    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        let handler = self.mouse_motion.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let x = Value::Float(x as f64);
        let y = Value::Float(y as f64);
        let dx = Value::Float(dx as f64);
        let dy = Value::Float(dy as f64);
        self.call_handler("mouse_motion", handler, ctx, vec![x, y, dx, dy]);
    }
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let handler = self.mouse_wheel.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let x = Value::Float(x as f64);
        let y = Value::Float(y as f64);
        self.call_handler("mouse_wheel", handler, ctx, vec![x, y]);
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        let handler = self.key_down.borrow().clone();
        if let Value::Nil = &handler {
            // Default behavior is just to quit on Escape
            if let KeyCode::Escape = keycode {
                event::quit(ctx);
            }
            return;
        }
        let keycode = self.translate_keycode(keycode);
        let keymods = self.translate_keymods(keymods, repeat);
        self.call_handler("key_down", handler, ctx, vec![keycode, keymods.into()]);
    }
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        let handler = self.key_up.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let keycode = self.translate_keycode(keycode);
        let keymods = self.translate_keymods(keymods, false);
        self.call_handler("key_up", handler, ctx, vec![keycode, keymods.into()]);
    }
    fn text_input_event(&mut self, ctx: &mut Context, c: char) {
        let handler = self.text_input.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let ch_val = self.globals.char_to_val(c);
        self.call_handler("text_input", handler, ctx, vec![ch_val]);
    }
    fn gamepad_button_down_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
        let handler = self.gamepad_button_down.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let btn = self.translate_gamepad_button(btn);
        let id = Value::Int(self.shared.borrow_mut().gamepad.index(id));
        self.call_handler("gamepad_button_down", handler, ctx, vec![btn, id]);
    }
    fn gamepad_axis_event(&mut self, ctx: &mut Context, axis: Axis, value: f32, id: GamepadId) {
        let handler = self.gamepad_axis.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let axis = self.translate_axis(axis);
        let value = Value::Float(value as f64);
        let id = Value::Int(self.shared.borrow_mut().gamepad.index(id));
        self.call_handler("gamepad_axis", handler, ctx, vec![axis, value, id]);
    }
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let handler = self.resize.borrow().clone();
        if let Value::Nil = &handler {
            return;
        }
        let width = Value::Float(width as f64);
        let height = Value::Float(height as f64);
        self.call_handler("resize", handler, ctx, vec![width, height]);
    }
}

fn list_keycode_symbols(globals: &mut Globals) -> Vec<Symbol> {
    let mut ret = Vec::new();
    for keycode in list_keycodes() {
        ret.push(globals.intern_str(&format!("{:?}", keycode)));
    }
    ret.into()
}

fn list_keycodes() -> Vec<KeyCode> {
    // TOOD: Figure out how to do this without having to copy and
    // paste the entire enum from ggez
    let keycodes = vec![
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
        KeyCode::Key9,
        KeyCode::Key0,
        KeyCode::A,
        KeyCode::B,
        KeyCode::C,
        KeyCode::D,
        KeyCode::E,
        KeyCode::F,
        KeyCode::G,
        KeyCode::H,
        KeyCode::I,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::M,
        KeyCode::N,
        KeyCode::O,
        KeyCode::P,
        KeyCode::Q,
        KeyCode::R,
        KeyCode::S,
        KeyCode::T,
        KeyCode::U,
        KeyCode::V,
        KeyCode::W,
        KeyCode::X,
        KeyCode::Y,
        KeyCode::Z,
        KeyCode::Escape,
        KeyCode::F1,
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::F4,
        KeyCode::F5,
        KeyCode::F6,
        KeyCode::F7,
        KeyCode::F8,
        KeyCode::F9,
        KeyCode::F10,
        KeyCode::F11,
        KeyCode::F12,
        KeyCode::F13,
        KeyCode::F14,
        KeyCode::F15,
        KeyCode::F16,
        KeyCode::F17,
        KeyCode::F18,
        KeyCode::F19,
        KeyCode::F20,
        KeyCode::F21,
        KeyCode::F22,
        KeyCode::F23,
        KeyCode::F24,
        KeyCode::Snapshot,
        KeyCode::Scroll,
        KeyCode::Pause,
        KeyCode::Insert,
        KeyCode::Home,
        KeyCode::Delete,
        KeyCode::End,
        KeyCode::PageDown,
        KeyCode::PageUp,
        KeyCode::Left,
        KeyCode::Up,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Back,
        KeyCode::Return,
        KeyCode::Space,
        KeyCode::Compose,
        KeyCode::Caret,
        KeyCode::Numlock,
        KeyCode::Numpad0,
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
        KeyCode::Numpad7,
        KeyCode::Numpad8,
        KeyCode::Numpad9,
        KeyCode::AbntC1,
        KeyCode::AbntC2,
        KeyCode::Add,
        KeyCode::Apostrophe,
        KeyCode::Apps,
        KeyCode::At,
        KeyCode::Ax,
        KeyCode::Backslash,
        KeyCode::Calculator,
        KeyCode::Capital,
        KeyCode::Colon,
        KeyCode::Comma,
        KeyCode::Convert,
        KeyCode::Decimal,
        KeyCode::Divide,
        KeyCode::Equals,
        KeyCode::Grave,
        KeyCode::Kana,
        KeyCode::Kanji,
        KeyCode::LAlt,
        KeyCode::LBracket,
        KeyCode::LControl,
        KeyCode::LShift,
        KeyCode::LWin,
        KeyCode::Mail,
        KeyCode::MediaSelect,
        KeyCode::MediaStop,
        KeyCode::Minus,
        KeyCode::Multiply,
        KeyCode::Mute,
        KeyCode::MyComputer,
        KeyCode::NavigateForward,  // also called "Prior"
        KeyCode::NavigateBackward, // also called "Next"
        KeyCode::NextTrack,
        KeyCode::NoConvert,
        KeyCode::NumpadComma,
        KeyCode::NumpadEnter,
        KeyCode::NumpadEquals,
        KeyCode::OEM102,
        KeyCode::Period,
        KeyCode::PlayPause,
        KeyCode::Power,
        KeyCode::PrevTrack,
        KeyCode::RAlt,
        KeyCode::RBracket,
        KeyCode::RControl,
        KeyCode::RShift,
        KeyCode::RWin,
        KeyCode::Semicolon,
        KeyCode::Slash,
        KeyCode::Sleep,
        KeyCode::Stop,
        KeyCode::Subtract,
        KeyCode::Sysrq,
        KeyCode::Tab,
        KeyCode::Underline,
        KeyCode::Unlabeled,
        KeyCode::VolumeDown,
        KeyCode::VolumeUp,
        KeyCode::Wake,
        KeyCode::WebBack,
        KeyCode::WebFavorites,
        KeyCode::WebForward,
        KeyCode::WebHome,
        KeyCode::WebRefresh,
        KeyCode::WebSearch,
        KeyCode::WebStop,
        KeyCode::Yen,
        KeyCode::Copy,
        KeyCode::Paste,
        KeyCode::Cut,
    ];
    for (i, keycode) in keycodes.iter().enumerate() {
        assert_eq!(i, *keycode as usize);
    }
    keycodes
}

fn list_gamepad_buttons(globals: &mut Globals) -> HashMap<Button, Symbol> {
    vec![
        (Button::South, globals.intern_str("South")),
        (Button::East, globals.intern_str("East")),
        (Button::North, globals.intern_str("North")),
        (Button::West, globals.intern_str("West")),
        (Button::C, globals.intern_str("C")),
        (Button::Z, globals.intern_str("Z")),
        (
            Button::LeftTrigger,
            globals.intern_str("LeftTrigger").into(),
        ),
        (
            Button::LeftTrigger2,
            globals.intern_str("LeftTrigger2").into(),
        ),
        (
            Button::RightTrigger,
            globals.intern_str("RightTrigger").into(),
        ),
        (
            Button::RightTrigger2,
            globals.intern_str("RightTrigger2").into(),
        ),
        (Button::Select, globals.intern_str("Select")),
        (Button::Start, globals.intern_str("Start")),
        (Button::Mode, globals.intern_str("Mode")),
        (Button::LeftThumb, globals.intern_str("LeftThumb")),
        (Button::RightThumb, globals.intern_str("RightThumb")),
        (Button::DPadUp, globals.intern_str("DPadUp")),
        (Button::DPadDown, globals.intern_str("DPadDown")),
        (Button::DPadLeft, globals.intern_str("DPadLeft")),
        (Button::DPadRight, globals.intern_str("DPadRight")),
        (Button::Unknown, globals.intern_str("Unknown")),
    ]
    .into_iter()
    .collect()
}

fn list_gamepad_axes(globals: &mut Globals) -> HashMap<Axis, Symbol> {
    vec![
        (Axis::LeftStickX, globals.intern_str("LeftStickX")),
        (Axis::LeftStickY, globals.intern_str("LeftStickY")),
        (Axis::LeftZ, globals.intern_str("LeftZ")),
        (Axis::RightStickX, globals.intern_str("RightStickX")),
        (Axis::RightStickY, globals.intern_str("RightStickY")),
        (Axis::RightZ, globals.intern_str("RightZ")),
        (Axis::DPadX, globals.intern_str("DPadX")),
        (Axis::DPadY, globals.intern_str("DPadY")),
        (Axis::Unknown, globals.intern_str("Unknown")),
    ]
    .into_iter()
    .collect()
}
