//! Functions for dealing with graphics
use super::to_wctx;
use super::with_wctx;
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

pub const NAME: &str = "a.gg._ngr";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::sdnew0(sr, "size", &["ctx"], Some(concat!(
                "Returns the size of the window's underlying drawable in pixels as [width, height].\n",
                "Returns zeros if window doesn't exist.",
            )),|globals, args, _kwargs| {
                let ctx = to_wctx(globals, &args[0])?;
                let (width, height) = graphics::drawable_size(ctx.get());
                Ok(vec![Value::Float(width as f64), Value::Float(height as f64)].into())
            }),
            NativeFunction::sdnew0(sr, "set_fullscreen", &["ctx", "type"], Some(concat!(
                "Sets the window to fullscreen or back\n",
                "type = 0 implies windowed mode\n",
                "type = 1 implies true fullscreen\n",
                "  used to be preferred 'cause it can have small performance\n",
                "  benefits over windowed fullscreen\n",
                "type = 2 implies windowed fullscreen\n",
                "  generally preferred over real fullscreen these days\n",
                "  'cause it plays nicer with multiple monitors\n",
            )),|globals, args, _kwargs| {
                let ctx = to_wctx(globals, &args[0])?;
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
