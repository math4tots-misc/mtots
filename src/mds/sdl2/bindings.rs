use crate::sdl2;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::Opaque;
use crate::Stashable;
use crate::Symbol;
use crate::Value;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use std::cell::Ref;
use super::keycode_to_key;
use super::KEY_COUNT;
use std::cell::RefMut;

pub(super) fn from_window(window: Window) -> Value {
    Opaque::new(window).into()
}

// pub(super) fn to_window_mut<'a>(
//     globals: &mut Globals,
//     value: &'a Value,
// ) -> EvalResult<RefMut<'a, Window>> {
//     Eval::expect_opaque_mut(globals, value)
// }

// pub(super) fn to_window<'a>(
//     globals: &mut Globals,
//     value: &'a Value,
// ) -> EvalResult<Ref<'a, Window>> {
//     Eval::expect_opaque(globals, value)
// }

pub(super) fn move_window<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Window> {
    Eval::move_opaque(globals, value)
}

pub(super) fn from_canvas(canvas: WindowCanvas) -> Value {
    Opaque::new(canvas).into()
}

pub(super) fn to_canvas_mut<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<RefMut<'a, WindowCanvas>> {
    Eval::expect_opaque_mut(globals, value)
}

pub(super) fn to_canvas<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<Ref<'a, WindowCanvas>> {
    Eval::expect_opaque(globals, value)
}

pub(super) fn to_color(globals: &mut Globals, value: &Value) -> EvalResult<Color> {
    if Eval::expect_list(globals, value)?.len() == 4 {
        let (r, g, b, a) = Eval::unpack4(globals, value)?;
        let r = Eval::expect_u8(globals, &r)?;
        let g = Eval::expect_u8(globals, &g)?;
        let b = Eval::expect_u8(globals, &b)?;
        let a = Eval::expect_u8(globals, &a)?;
        Ok(Color::RGBA(r, g, b, a))
    } else {
        let (r, g, b) = Eval::unpack_triple(globals, value)?;
        let r = Eval::expect_u8(globals, &r)?;
        let g = Eval::expect_u8(globals, &g)?;
        let b = Eval::expect_u8(globals, &b)?;
        Ok(Color::RGB(r, g, b))
    }
}

pub(super) fn to_rect(globals: &mut Globals, value: &Value) -> EvalResult<Rect> {
    let (x, y, width, height) = Eval::unpack4(globals, value)?;
    let x = Eval::expect_i32(globals, &x)?;
    let y = Eval::expect_i32(globals, &y)?;
    let width = Eval::expect_u32(globals, &width)?;
    let height = Eval::expect_u32(globals, &height)?;
    Ok(Rect::new(x, y, width, height))
}

struct EventSymbols {
    // quit: Option<Symbol>,
    text: Option<Symbol>,
    keydown: Option<Symbol>,
    keyup: Option<Symbol>,
    mousedown: Option<Symbol>,
    mouseup: Option<Symbol>,
    keycodes: Vec<Option<Symbol>>,
    mousebtns: Vec<Option<Symbol>>,
}

impl Default for EventSymbols {
    fn default() -> Self {
        Self {
            // quit: None,
            text: None,
            keydown: None,
            keyup: None,
            mousedown: None,
            mouseup: None,
            keycodes: {
                let mut codes = vec![];
                codes.resize_with(KEY_COUNT, || None);
                codes
            },
            mousebtns: {
                let mut codes = vec![];
                codes.resize_with(MouseButton::X2 as usize + 1, || None);
                codes
            },
        }
    }
}

impl Stashable for EventSymbols {}

impl EventSymbols {
    // pub fn quit(&mut self, globals: &mut Globals) -> Symbol {
    //     if self.quit.is_none() {
    //         self.quit = Some(globals.intern_str("quit"));
    //     }
    //     self.quit.unwrap()
    // }
    pub fn text(&mut self, globals: &mut Globals) -> Symbol {
        if self.text.is_none() {
            self.text = Some(globals.intern_str("Text"));
        }
        self.text.unwrap()
    }
    pub fn keydown(&mut self, globals: &mut Globals) -> Symbol {
        if self.keydown.is_none() {
            self.keydown = Some(globals.intern_str("KeyDown"));
        }
        self.keydown.unwrap()
    }
    pub fn keyup(&mut self, globals: &mut Globals) -> Symbol {
        if self.keyup.is_none() {
            self.keyup = Some(globals.intern_str("KeyUp"));
        }
        self.keyup.unwrap()
    }
    pub fn mousedown(&mut self, globals: &mut Globals) -> Symbol {
        if self.mousedown.is_none() {
            self.mousedown = Some(globals.intern_str("MouseDown"));
        }
        self.mousedown.unwrap()
    }
    pub fn mouseup(&mut self, globals: &mut Globals) -> Symbol {
        if self.mouseup.is_none() {
            self.mouseup = Some(globals.intern_str("MouseDown"));
        }
        self.mouseup.unwrap()
    }
    pub fn keycode(&mut self, globals: &mut Globals, keycode: Keycode) -> Symbol {
        let i = keycode_to_key(keycode);
        if self.keycodes[i].is_none() {
            self.keycodes[i] = Some(globals.intern_str(&format!("{:?}", keycode)));
        }
        self.keycodes[i].unwrap()
    }
    pub fn mousebtn(&mut self, globals: &mut Globals, btn: MouseButton) -> Symbol {
        let i = btn as usize;
        if self.mousebtns[i].is_none() {
            self.mousebtns[i] = Some(globals.intern_str(&format!("{:?}", btn)));
        }
        self.mousebtns[i].unwrap()
    }
}

pub(super) fn from_events(globals: &mut Globals, events: Vec<Event>) -> EvalResult<Vec<Value>> {
    let mut vec = Vec::<Value>::new();
    let es = globals.get_from_stash::<EventSymbols>();
    for event in events {
        match event {
            Event::Quit { timestamp: _ } => {
                return globals.set_exc_str("quit")?;
            }
            Event::TextInput {
                timestamp: _,
                window_id,
                text,
            } => {
                let mut ev = Vec::<Value>::new();
                ev.push(es.borrow_mut().text(globals).into());
                ev.push((window_id as i64).into());
                ev.push(text.into());
                vec.push(ev.into());
            }
            Event::KeyDown {
                timestamp: _,
                window_id,
                keycode,
                scancode,
                keymod,
                repeat,
            } => {
                let mut ev = Vec::<Value>::new();
                ev.push(es.borrow_mut().keydown(globals).into());
                ev.push((window_id as i64).into());
                ev.push(if let Some(keycode) = keycode {
                    es.borrow_mut().keycode(globals, keycode).into()
                } else {
                    Value::Nil
                });
                ev.push(if let Some(scancode) = scancode {
                    (scancode as i64).into()
                } else {
                    Value::Nil
                });
                ev.push((keymod.bits() as i64).into());
                ev.push(repeat.into());
                vec.push(ev.into());
            }
            Event::KeyUp {
                timestamp: _,
                window_id,
                keycode,
                scancode,
                keymod,
                repeat,
            } => {
                let mut ev = Vec::<Value>::new();
                ev.push(es.borrow_mut().keyup(globals).into());
                ev.push((window_id as i64).into());
                ev.push(if let Some(keycode) = keycode {
                    es.borrow_mut().keycode(globals, keycode).into()
                } else {
                    Value::Nil
                });
                ev.push(if let Some(scancode) = scancode {
                    (scancode as i64).into()
                } else {
                    Value::Nil
                });
                ev.push((keymod.bits() as i64).into());
                ev.push(repeat.into());
                vec.push(ev.into());
            }
            Event::MouseButtonDown {
                timestamp: _,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                let mut ev = Vec::<Value>::new();
                ev.push(es.borrow_mut().mousedown(globals).into());
                ev.push((window_id as i64).into());
                ev.push((which as i64).into());
                ev.push(es.borrow_mut().mousebtn(globals, mouse_btn).into());
                ev.push((clicks as i64).into());
                ev.push((x as i64).into());
                ev.push((y as i64).into());
                vec.push(ev.into());
            }
            Event::MouseButtonUp {
                timestamp: _,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                let mut ev = Vec::<Value>::new();
                ev.push(es.borrow_mut().mouseup(globals).into());
                ev.push((window_id as i64).into());
                ev.push((which as i64).into());
                ev.push(es.borrow_mut().mousebtn(globals, mouse_btn).into());
                ev.push((clicks as i64).into());
                ev.push((x as i64).into());
                ev.push((y as i64).into());
                vec.push(ev.into());
            }
            _ => {}
        }
    }
    Ok(vec)
}
