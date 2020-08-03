use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Stashable;
use crate::Value;
use ggez::graphics::Color;
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
}

impl ggez::event::EventHandler for EventHandler {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(update) = self.update.clone() {
            let r = Eval::call(&mut self.globals, &update, vec![]);
            ordie(&mut self.globals, r);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if let Some(draw) = self.draw.clone() {
            let r = Eval::call(&mut self.globals, &draw, vec![]);
            ordie(&mut self.globals, r);
            ggez::graphics::present(ctx)?;
        }
        std::thread::yield_now();
        Ok(())
    }
}

struct Stash {
    ctx: &'static mut ggez::Context,
}

impl Stashable for Stash {}

pub(super) fn load(_globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::snew(
                "run",
                (
                    &["name", "author", "init", "update", "draw"],
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
