//! JSON bindings
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Value;
use crate::Opaque;
use crate::ErrorIndicator;
use std::cell::RefCell;
use std::cell::Ref;
use std::collections::HashMap;
use std::rc::Rc;
use ggez::graphics;
use ggez::Context;
use ggez::ContextBuilder;
use ggez::GameResult;
use ggez::GameError;
use ggez::event;
use ggez::event::EventHandler;
use ggez::graphics::MeshBuilder;
use ggez::graphics::Mesh;
use ggez::graphics::Color;

pub const NAME: &str = "_ggez";

type Point = ggez::mint::Point2<f32>;

fn mkpt(x: f32, y: f32) -> Point {
    Point { x, y }
}

fn try_<R>(globals: &mut Globals, r: GameResult<R>) -> EvalResult<R> {
    match r {
        Ok(r) => Ok(r),
        Err(error) => globals.set_exc_str(&format!("{:?}", error)),
    }
}

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
        NativeFunction::simple0(
            sr,
            "new_color",
            &["r", "g", "b", "a"],
            |globals, args, _kwargs| {
                let r = Eval::expect_floatlike(globals, &args[0])? as f32;
                let g = Eval::expect_floatlike(globals, &args[1])? as f32;
                let b = Eval::expect_floatlike(globals, &args[2])? as f32;
                let a = Eval::expect_floatlike(globals, &args[3])? as f32;
                from_color(globals, Color { r, g, b, a })
            },
        ),
        NativeFunction::simple0(
            sr,
            "new_mesh_builder",
            &[],
            |globals, _args, _kwargs| {
                from_mesh_builder(globals, MeshBuilder::new())
            },
        ),
        NativeFunction::simple0(
            sr,
            "mesh_builder_circle",
            &["mesh_builder", "center", "radius", "color"],
            |globals, args, _kwargs| {
                let mesh_builder = to_mesh_builder_ref(globals, &args[0])?;
                let center = expect_point(globals, &args[1])?;
                let radius = Eval::expect_floatlike(globals, &args[2])? as f32;
                let color = to_color_ref(globals, &args[3])?.clone();
                mesh_builder.borrow_mut().circle(
                    graphics::DrawMode::fill(),
                    center,
                    radius,
                    2.0,  // tolerance
                    color,
                );
                Ok(Value::Nil)
            },
        ),
        NativeFunction::simple0(
            sr,
            "mesh_builder_build",
            &["mesh_builder", "ctx"],
            |globals, args, _kwargs| {
                let mesh_builder = to_mesh_builder(globals, &args[0])?;
                let ctx_refcell = to_ctx(globals, &args[1])?;
                let mut ctx = ctx_refcell.borrow_mut();
                let mesh = try_(globals, mesh_builder.build(ctx.get_mut()))?;
                from_mesh(globals, mesh)
            },
        ),
        NativeFunction::simple0(
            sr,
            "draw",
            &["ctx", "drawable"],
            |globals, args, _kwargs| {
                let ctx_refcell = to_ctx(globals, &args[0])?;
                let mut ctx = ctx_refcell.borrow_mut();
                let drawable = to_drawable(globals, &args[1])?;
                try_(globals, draw(ctx.get_mut(), &drawable))?;
                Ok(Value::Nil)
            },
        ),
        NativeFunction::simple0(
            sr,
            "size",
            &["ctx"],
            |globals, args, _kwargs| {
                let ctx_refcell = to_ctx(globals, &args[0])?;
                let ctx = ctx_refcell.borrow();
                let (width, height) = graphics::drawable_size(ctx.get());
                Ok(vec![Value::Float(width as f64), Value::Float(height as f64)].into())
            },
        ),
        NativeFunction::simple0(
            sr,
            "start",
            &["name", "author", "init", "update", "draw"],
            |globals, args, _kwargs| {
                struct State<'a> {
                    globals: &'a mut Globals,
                    update: &'a Value,
                    draw: &'a Value,
                }

                impl<'a> State<'a> {
                    fn new(globals: &'a mut Globals, _ctx: &mut Context, update: &'a Value, draw: &'a Value) -> State<'a> {
                        State {
                            globals,
                            update,
                            draw,
                        }
                    }

                    fn err(&self) -> Option<GameError> {
                        if self.globals.exc_occurred() {
                            Some(GameError::EventLoopError("Script error".to_owned()))
                        } else {
                            None
                        }
                    }
                }

                impl<'a> EventHandler for State<'a> {
                    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
                        if let Some(e) = self.err() {
                            return Err(e);
                        }
                        let update = self.update;
                        if !update.is_nil() {
                            to_game_result(with_ctx(self.globals, ctx, |globals, ctx_val| {
                                Eval::call(globals, update, vec![ctx_val.clone()])
                            }))?;
                        }
                        Ok(())
                    }

                    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
                        if let Some(e) = self.err() {
                            return Err(e);
                        }
                        graphics::clear(ctx, graphics::BLACK);
                        let draw = self.draw;
                        if !draw.is_nil() {
                            to_game_result(with_ctx(self.globals, ctx, |globals, ctx_val| {
                                Eval::call(globals, draw, vec![ctx_val.clone()])
                            }))?;
                        }
                        graphics::present(ctx)
                    }
                }

                let name = Eval::expect_string(globals, &args[0])?;
                let author = Eval::expect_string(globals, &args[1])?;
                let (mut ctx, mut event_loop) = ContextBuilder::new(name, author)
                    .build()
                    .unwrap();
                let mut state = State::new(globals, &mut ctx, &args[3], &args[4]);

                // call 'init'
                if !args[2].is_nil() {
                    with_ctx(state.globals, &mut ctx, |globals, ctx_val| {
                        Eval::call(globals, &args[2], vec![ctx_val.clone()])
                    })?;
                }

                match event::run(&mut ctx, &mut event_loop, &mut state) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => if globals.exc_occurred() {
                        Err(ErrorIndicator)
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

mod wctx {
    // I need to somehow expose the 'Context' variable to the script itself
    // I couldn't figure out a way to do it without any unsafe code, but at least
    // I think I can limit the unsafe to this mod block.
    use super::*;

    pub(super) struct WrappedContext {
        ctx: &'static mut Context,
    }

    impl WrappedContext {
        pub fn get_mut(&mut self) -> &mut Context {
            self.ctx
        }
        pub fn get(&self) -> &Context {
            self.ctx
        }
    }

    pub(super) fn to_ctx<'a>(globals: &mut Globals, v: &'a Value) -> EvalResult<Ref<'a, RefCell<WrappedContext>>> {
        Eval::expect_opaque(globals, v)
    }

    pub(super) fn with_ctx<F, R>(globals: &mut Globals, ctx: &mut Context, f: F) -> EvalResult<R>
    where F: FnOnce(&mut Globals, &Value) -> EvalResult<R>
    {
        let wctx = WrappedContext {
            ctx: unsafe { std::mem::transmute::<&'_ mut Context, &'static mut Context>(ctx) },
        };
        let refcell: RefCell<WrappedContext> = RefCell::new(wctx);
        let opaque = Opaque::new(refcell);
        let value: Value = opaque.into();

        let r = f(globals, &value);

        // Make sure to remove the WrappedContext object so that it's no longer
        // accessible from the script (e.g. through any lingering references)
        let _: RefCell<WrappedContext> = Eval::move_opaque(globals, &value).unwrap();

        r
    }
}

// use wctx::WrappedContext;
use wctx::to_ctx;
use wctx::with_ctx;

fn expect_point(globals: &mut Globals, point: &Value) -> EvalResult<Point> {
    let (x, y) = Eval::unpack_pair(globals, point)?;
    let x = Eval::expect_floatlike(globals, &x)?;
    let y = Eval::expect_floatlike(globals, &y)?;
    Ok(mkpt(x as f32, y as f32))
}

fn from_color(_globals: &mut Globals, color: Color) -> EvalResult<Value> {
    let opaque = Opaque::new(color);
    Ok(opaque.into())
}

fn to_color_ref<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, Color>> {
    Eval::expect_opaque(globals, value)
}

fn to_game_result<R>(eval_result: EvalResult<R>) -> GameResult<R> {
    match eval_result {
        Ok(r) => Ok(r),
        Err(_) => Err(GameError::EventLoopError("script error".to_owned())),
    }
}

fn from_mesh_builder(_globals: &mut Globals, mesh_builder: MeshBuilder) -> EvalResult<Value> {
    let mesh_builder: RefCell<MeshBuilder> = RefCell::new(mesh_builder);
    let opaque = Opaque::new(mesh_builder);
    Ok(opaque.into())
}

fn to_mesh_builder_ref<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<
Ref<'a, RefCell<MeshBuilder>>
> {
    Eval::expect_opaque(globals, value)
}

fn to_mesh_builder(globals: &mut Globals, value: &Value) -> EvalResult<MeshBuilder> {
    let refcell: RefCell<MeshBuilder> = Eval::move_opaque(globals, value)?;
    Ok(refcell.into_inner())
}

fn from_mesh(_globals: &mut Globals, mesh: Mesh) -> EvalResult<Value> {
    let drawable: EDrawable = EDrawable::Mesh(mesh);
    let opaque = Opaque::new(drawable);
    Ok(opaque.into())
}

fn to_drawable<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, EDrawable>> {
    Eval::expect_opaque(globals, value)
}

enum EDrawable {
    Mesh(Mesh),
}

fn draw(ctx: &mut Context, drawable: &EDrawable) -> GameResult<()> {
    match drawable {
        EDrawable::Mesh(mesh) => graphics::draw(ctx, mesh, graphics::DrawParam::default()),
    }
}
