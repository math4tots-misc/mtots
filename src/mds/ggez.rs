//! JSON bindings
use crate::ErrorIndicator;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Opaque;
use crate::RcStr;
use crate::Value;
use crate::Symbol;
use ggez::event;
use ggez::event::EventHandler;
use ggez::event::MouseButton;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::Mesh;
use ggez::graphics::MeshBuilder;
use ggez::Context;
use ggez::ContextBuilder;
use ggez::GameError;
use ggez::GameResult;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
            NativeFunction::simple0(sr, "new_mesh_builder", &[], |globals, _args, _kwargs| {
                from_mesh_builder(globals, MeshBuilder::new())
            }),
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
                        2.0, // tolerance
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
                &["ctx", "drawable", "destination"],
                |globals, args, _kwargs| {
                    let ctx_refcell = to_ctx(globals, &args[0])?;
                    let mut ctx = ctx_refcell.borrow_mut();
                    let drawable = to_drawable(globals, &args[1])?;
                    let destination = expect_point(globals, &args[2])?;
                    try_(globals, draw(ctx.get_mut(), &drawable, destination))?;
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(sr, "size", &["ctx"], |globals, args, _kwargs| {
                let ctx_refcell = to_ctx(globals, &args[0])?;
                let ctx = ctx_refcell.borrow();
                let (width, height) = graphics::drawable_size(ctx.get());
                Ok(vec![Value::Float(width as f64), Value::Float(height as f64)].into())
            }),
            NativeFunction::simple0(
                sr,
                "start",
                &["name", "author", "sleep_per_frame", "init", "update", "draw", "mouse_down"],
                |globals, args, _kwargs| {
                    struct State<'a> {
                        globals: &'a mut Globals,
                        update: &'a Value,
                        draw: &'a Value,
                        mouse_down: &'a Value,

                        sleep_per_frame: Option<std::time::Duration>,

                        symbol_left: Symbol,
                        symbol_right: Symbol,
                        symbol_middle: Symbol,
                    }

                    impl<'a> State<'a> {
                        fn new(
                            globals: &'a mut Globals,
                            _ctx: &mut Context,
                            sleep_per_frame: Option<std::time::Duration>,
                            update: &'a Value,
                            draw: &'a Value,
                            mouse_down: &'a Value,
                        ) -> State<'a> {
                            let symbol_left = globals.intern_str("left");
                            let symbol_right = globals.intern_str("right");
                            let symbol_middle = globals.intern_str("middle");
                            State {
                                globals,
                                sleep_per_frame,
                                update,
                                draw,
                                mouse_down,
                                symbol_left,
                                symbol_right,
                                symbol_middle,
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
                            graphics::present(ctx)?;
                            if let Some(dur) = self.sleep_per_frame {
                                std::thread::sleep(dur)
                            } else {
                                ggez::timer::yield_now();
                            }
                            Ok(())
                        }

                        fn mouse_button_down_event(
                            &mut self,
                            ctx: &mut Context,
                            button: MouseButton,
                            x: f32,
                            y: f32,
                        ) {
                            if let Some(_) = self.err() {
                                return;
                            }
                            let mouse_down = self.mouse_down;
                            if !mouse_down.is_nil() {
                                let button: Value = match button {
                                    MouseButton::Left => self.symbol_left.into(),
                                    MouseButton::Right => self.symbol_right.into(),
                                    MouseButton::Middle => self.symbol_middle.into(),
                                    MouseButton::Other(i) => Value::Int(i as i64),
                                };
                                let x = Value::Float(x as f64);
                                let y = Value::Float(y as f64);
                                let _r = with_ctx(self.globals, ctx, |globals, ctx_val| {
                                    Eval::call(globals, mouse_down, vec![ctx_val.clone(), button, x, y])
                                });
                            }
                        }
                    }

                    let name = Eval::expect_string(globals, &args[0])?;
                    let author = Eval::expect_string(globals, &args[1])?;
                    let sleep_per_frame_val = &args[2];
                    let sleep_per_frame = if let Value::Nil = sleep_per_frame_val {
                        None
                    } else {
                        Some(std::time::Duration::from_secs_f64(
                            Eval::expect_floatlike(globals, sleep_per_frame_val)?))
                    };
                    let (mut ctx, mut event_loop) =
                        ContextBuilder::new(name, author).build().unwrap();
                    let init = &args[3];
                    let mut state = State::new(globals, &mut ctx, sleep_per_frame, &args[4], &args[5], &args[6]);

                    // call 'init'
                    if !init.is_nil() {
                        with_ctx(state.globals, &mut ctx, |globals, ctx_val| {
                            Eval::call(globals, &init, vec![ctx_val.clone()])
                        })?;
                    }

                    match event::run(&mut ctx, &mut event_loop, &mut state) {
                        Ok(_) => Ok(Value::Nil),
                        Err(e) => {
                            if globals.exc_occurred() {
                                Err(ErrorIndicator)
                            } else {
                                globals.set_exc_str(&format!("{:?}", e))
                            }
                        }
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

    pub(super) fn to_ctx<'a>(
        globals: &mut Globals,
        v: &'a Value,
    ) -> EvalResult<Ref<'a, RefCell<WrappedContext>>> {
        Eval::expect_opaque(globals, v)
    }

    pub(super) fn with_ctx<F, R>(globals: &mut Globals, ctx: &mut Context, f: F) -> EvalResult<R>
    where
        F: FnOnce(&mut Globals, &Value) -> EvalResult<R>,
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

fn to_mesh_builder_ref<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<Ref<'a, RefCell<MeshBuilder>>> {
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

fn draw(ctx: &mut Context, drawable: &EDrawable, dest: Point) -> GameResult<()> {
    match drawable {
        EDrawable::Mesh(mesh) => {
            graphics::draw(ctx, mesh, graphics::DrawParam::default().dest(dest))
        }
    }
}
