//! JSON bindings
use crate::ErrorIndicator;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Opaque;
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
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::spritebatch::SpriteIdx;
use ggez::graphics::Canvas;
use ggez::graphics::Color;
use ggez::graphics::Image;
use ggez::graphics::Mesh;
use ggez::graphics::MeshBuilder;
use ggez::graphics::Scale;
use ggez::graphics::Text;
use ggez::graphics::TextFragment;
use ggez::Context;
use ggez::ContextBuilder;
use ggez::GameError;
use ggez::GameResult;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::rc::Rc;

pub const NAME: &str = "a.ggez.n._ctx";

type Point = ggez::mint::Point2<f32>;

fn mkpt(x: f32, y: f32) -> Point {
    Point { x, y }
}

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::simple0(sr, "ctx_size", &["ctx"], |globals, args, _kwargs| {
                let ctx_refcell = to_ctx(globals, &args[0])?;
                let ctx = ctx_refcell.borrow();
                let (width, height) = graphics::drawable_size(ctx.get());
                Ok(vec![Value::Float(width as f64), Value::Float(height as f64)].into())
            }),
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
