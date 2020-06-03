//! JSON bindings
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};

pub const NAME: &str = "_ggez";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![NativeFunction::simple0(
            sr,
            "sleep",
            &["sec"],
            |globals, args, _| {
                let secs = Eval::expect_floatlike(globals, &args[0])?;
                std::thread::sleep(std::time::Duration::from_secs_f64(secs));
                Ok(Value::Nil)
            },
        ),
        NativeFunction::simple0(
            sr,
            "start",
            &["callback_table"],
            |globals, _args, _kwargs| {
                struct State<'a> {
                    _globals: &'a mut Globals,
                }

                impl<'a> State<'a> {
                    pub fn new(_globals: &'a mut Globals, _ctx: &mut Context) -> State<'a> {
                        State {
                            _globals,
                        }
                    }
                }

                impl<'a> EventHandler for State<'a> {
                    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
                        Ok(())
                    }

                    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
                        graphics::clear(ctx, graphics::BLACK);
                        let circle = graphics::Mesh::new_circle(
                            ctx,
                            graphics::DrawMode::fill(),
                            ggez::nalgebra::Point2::new(200.0, 200.0),
                            100.0,
                            2.0,
                            graphics::WHITE,
                        )?;
                        graphics::draw(ctx, &circle, (ggez::nalgebra::Point2::new(0.0, 0.0), ))?;
                        graphics::present(ctx)
                    }
                }

                let (mut ctx, mut event_loop) = ContextBuilder::new("foo", "author")
                    .build()
                    .unwrap();
                let mut state = State::new(globals, &mut ctx);

                match event::run(&mut ctx, &mut event_loop, &mut state) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => if globals.exc_occurred() {
                        Err(mtots_core::ErrorIndicator)
                    } else {
                        globals.set_exc_str(&format!("{:?}", e))
                    },
                }
            },
        ),
        NativeFunction::simple0(
            sr,
            "main",
            &[],
            |globals, _args, _kwargs| {
                struct State<'a> {
                    _globals: &'a mut Globals,
                }

                impl<'a> State<'a> {
                    pub fn new(_globals: &'a mut Globals, _ctx: &mut Context) -> State<'a> {
                        State {
                            _globals,
                        }
                    }
                }

                impl<'a> EventHandler for State<'a> {
                    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
                        Ok(())
                    }

                    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
                        graphics::clear(ctx, graphics::BLACK);
                        let circle = graphics::Mesh::new_circle(
                            ctx,
                            graphics::DrawMode::fill(),
                            ggez::nalgebra::Point2::new(200.0, 200.0),
                            100.0,
                            2.0,
                            graphics::WHITE,
                        )?;
                        graphics::draw(ctx, &circle, (ggez::nalgebra::Point2::new(0.0, 0.0), ))?;
                        graphics::present(ctx)
                    }
                }

                let (mut ctx, mut event_loop) = ContextBuilder::new("foo", "author")
                    .build()
                    .unwrap();
                let mut state = State::new(globals, &mut ctx);

                match event::run(&mut ctx, &mut event_loop, &mut state) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => if globals.exc_occurred() {
                        Err(mtots_core::ErrorIndicator)
                    } else {
                        globals.set_exc_str(&format!("{:?}", e))
                    },
                }
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
