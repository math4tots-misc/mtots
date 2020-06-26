//! JSON bindings
use crate::anyhow::Result;
use crate::yagl::AppContext;
use crate::yagl::Color;
use crate::yagl::DeviceId;
use crate::yagl::Instance;
use crate::yagl::Key;
use crate::yagl::Rect;
use crate::yagl::RenderContext;
use crate::yagl::SpriteBatch;
use crate::yagl::SpriteSheet;
use crate::yagl::TextGrid;
use crate::Class;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Opaque;
use crate::RcStr;
use crate::Value;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::rc::Rc;

mod actx;

use actx::ACtx;

pub const NAME: &str = "a._yagl";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::sdnew0(
                sr,
                "run",
                &[
                    "ctx_class",
                    "init",
                    "update",
                    "render",
                    "resize",
                    "char",
                    "key_pressed",
                    "key_released",
                ],
                Some(concat!(
                    "Starts yagl's event loop.\n",
                    "ctx_class argument is the script-land class to wrap the ",
                    "raw AppContext object.\n",
                    "All remaining arguments are callbacks wrapped in Cell objects.\n",
                    "The Cells may contain nil, in which case it will be ignored.\n",
                    "Using Cell objects allows the script to swap them ",
                    "out as needed while the program is running.\n\n",
                    "The init argument is treated specially in that whenever it ",
                    "is called, it is set to nil. This is so that init is generally ",
                    "called at most once whenever set.\n",
                )),
                |globals, args, _kwargs| {
                    let mut args = args.into_iter();
                    let ctx_class = Eval::expect_class(globals, &args.next().unwrap())?.clone();
                    let init = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    let update = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    let render = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    let resize = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    let char = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    let key_pressed = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    let key_released = Eval::expect_cell(globals, &args.next().unwrap())?.clone();
                    assert!(args.next().is_none());
                    globals.escape_to_trampoline(move |globals| {
                        struct State {
                            globals: Globals,
                            ctx_class: Rc<Class>,
                            init: Rc<RefCell<Value>>,
                            update: Rc<RefCell<Value>>,
                            render: Rc<RefCell<Value>>,
                            resize: Rc<RefCell<Value>>,
                            char: Rc<RefCell<Value>>,
                            key_pressed: Rc<RefCell<Value>>,
                            key_released: Rc<RefCell<Value>>,
                        }
                        impl State {
                            fn call_handler(
                                &mut self,
                                actx: &mut AppContext,
                                name: &'static str,
                                handler: Value,
                                args: Vec<Value>,
                            ) -> Value {
                                let result = ACtx::call_handler(
                                    &mut self.globals,
                                    &handler,
                                    &self.ctx_class,
                                    actx,
                                    args,
                                );
                                match result {
                                    Ok(value) => value,
                                    Err(_) => {
                                        assert!(self.globals.print_if_error());
                                        panic!(
                                            "Uncaught exception while running handling {} event",
                                            name
                                        );
                                    }
                                }
                            }

                            #[allow(unused)]
                            fn translate_device(&mut self, dev: DeviceId) -> EvalResult<Value> {
                                // TODO
                                Ok(Value::Nil)
                            }

                            fn translate_key(&mut self, key: Key) -> EvalResult<Value> {
                                Ok(self.globals.intern_str(&format!("{:?}", key)).into())
                            }
                        }
                        impl yagl::Game for State {
                            fn update(&mut self, actx: &mut AppContext) -> Result<()> {
                                let handler = self.init.borrow().clone();
                                if !handler.is_nil() {
                                    self.call_handler(actx, "init", handler, vec![]);
                                    self.init.replace(Value::Nil);
                                }

                                let handler = self.update.borrow().clone();
                                if !handler.is_nil() {
                                    self.call_handler(actx, "update", handler, vec![]);
                                }
                                Ok(())
                            }

                            fn render(&mut self, rctx: &mut RenderContext) -> Result<()> {
                                let handler = self.render.borrow().clone();
                                if !handler.is_nil() {
                                    let list_val =
                                        self.call_handler(rctx.actx(), "render", handler, vec![]);
                                    let list = {
                                        let r = Eval::expect_list(&mut self.globals, &list_val);
                                        unwr(&mut self.globals, r)
                                    };
                                    let mut refs = Vec::new();
                                    for val in list.iter() {
                                        if is_text_grid(val) {
                                            refs.push({
                                                let r = to_text_grid(&mut self.globals, val);
                                                Ref::map(unwr(&mut self.globals, r), |tg| {
                                                    tg.batch()
                                                })
                                            });
                                        } else {
                                            refs.push({
                                                let r = to_batch(&mut self.globals, val);
                                                unwr(&mut self.globals, r)
                                            });
                                        }
                                    }
                                    let raw_refs: Vec<_> = refs
                                        .iter()
                                        .map(|r| {
                                            let r: &SpriteBatch = r;
                                            r
                                        })
                                        .collect();
                                    rctx.render(&raw_refs);
                                }
                                Ok(())
                            }

                            fn resize(
                                &mut self,
                                actx: &mut AppContext,
                                width: u32,
                                height: u32,
                            ) -> Result<()> {
                                let handler = self.resize.borrow().clone();
                                if !handler.is_nil() {
                                    let width = (width as i64).into();
                                    let height = (height as i64).into();
                                    self.call_handler(actx, "resize", handler, vec![width, height]);
                                }
                                Ok(())
                            }

                            fn ch(&mut self, actx: &mut AppContext, ch: char) -> Result<()> {
                                let handler = self.char.borrow().clone();
                                if !handler.is_nil() {
                                    let ch = self.globals.char_to_val(ch);
                                    self.call_handler(actx, "char", handler, vec![ch]);
                                }
                                Ok(())
                            }

                            fn key_pressed(
                                &mut self,
                                actx: &mut AppContext,
                                dev: DeviceId,
                                key: Key,
                            ) -> Result<()> {
                                let handler = self.key_pressed.borrow().clone();
                                if !handler.is_nil() {
                                    let dev = self.translate_device(dev).unwrap();
                                    let key = self.translate_key(key).unwrap();
                                    self.call_handler(actx, "key_pressed", handler, vec![dev, key]);
                                }
                                Ok(())
                            }

                            fn key_released(
                                &mut self,
                                actx: &mut AppContext,
                                dev: DeviceId,
                                key: Key,
                            ) -> Result<()> {
                                let handler = self.key_released.borrow().clone();
                                if !handler.is_nil() {
                                    let dev = self.translate_device(dev).unwrap();
                                    let key = self.translate_key(key).unwrap();
                                    self.call_handler(
                                        actx,
                                        "key_released",
                                        handler,
                                        vec![dev, key],
                                    );
                                }
                                Ok(())
                            }
                        }
                        let state = State {
                            globals,
                            ctx_class,
                            init,
                            update,
                            render,
                            resize,
                            char,
                            key_pressed,
                            key_released,
                        };
                        yagl::run(move |_actx| Ok(state))
                    })
                },
            ),
            NativeFunction::sdnew0(sr, "exit", &["ctx"], None, |globals, args, _kwargs| {
                let mut ctx = to_actx_mut(globals, &args[0])?;
                ctx.exit();
                Ok(Value::Nil)
            }),
            NativeFunction::sdnew0(sr, "scale", &["ctx"], None, |globals, args, _kwargs| {
                let ctx = to_actx(globals, &args[0])?;
                let [width, height] = ctx.scale();
                let width = Value::Float(width as f64);
                let height = Value::Float(height as f64);
                Ok(vec![width, height].into())
            }),
            NativeFunction::sdnew0(
                sr,
                "new_batch",
                &["ctx", "sheet"],
                None,
                |globals, args, _kwargs| {
                    let mut ctx = to_actx_mut(globals, &args[0])?;
                    let sheet = to_sheet(globals, &args[1])?.clone();
                    Ok(from_batch(match ctx.new_batch(sheet) {
                        Ok(batch) => batch,
                        Err(error) => return globals.set_exc_str(&format!("{:?}", error)),
                    }))
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "batch_len",
                &["batch"],
                None,
                |globals, args, _kwargs| {
                    let batch = to_batch(globals, &args[0])?;
                    Ok((batch.len() as i64).into())
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "batch_add",
                &["batch", "src", "dest", "rotate", "color_factor"],
                None,
                |globals, args, _kwargs| {
                    let mut batch = to_batch_mut(globals, &args[0])?;
                    let mut builder = Instance::builder();
                    if !args[1].is_nil() {
                        builder = builder.src(to_rect(globals, &args[1])?);
                    }
                    if !args[2].is_nil() {
                        builder = builder.dest(to_rect(globals, &args[2])?);
                    }
                    if !args[3].is_nil() {
                        builder = builder.rotate(to_f32(globals, &args[3])?);
                    }
                    if !args[4].is_nil() {
                        builder = builder.color_factor(to_color(globals, &args[4])?);
                    }
                    batch.add(builder);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "batch_set",
                &["batch", "i", "src", "dest", "rotate", "color_factor"],
                None,
                |globals, args, _kwargs| {
                    let mut batch = to_batch_mut(globals, &args[0])?;
                    let i = Eval::expect_usize(globals, &args[1])?;
                    let inst = batch.get_mut(i);
                    if !args[2].is_nil() {
                        inst.set_src(to_rect(globals, &args[2])?);
                    }
                    if !args[3].is_nil() {
                        inst.set_dest(to_rect(globals, &args[3])?);
                    }
                    if !args[4].is_nil() {
                        inst.set_rotation(to_f32(globals, &args[4])?);
                    }
                    if !args[5].is_nil() {
                        inst.set_color_factor(to_color(globals, &args[5])?);
                    }
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "new_sheet_from_color",
                &["ctx", "color"],
                None,
                |globals, args, _kwargs| {
                    let mut ctx = to_actx_mut(globals, &args[0])?;
                    let color = to_color(globals, &args[1])?;
                    Ok(from_sheet(try_(globals, ctx.new_sheet_from_color(color))?))
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "new_text_grid",
                &["ctx", "char_width", "nrows", "ncols"],
                None,
                |globals, args, _kwargs| {
                    let mut ctx = to_actx_mut(globals, &args[0])?;
                    let char_width = to_f32(globals, &args[1])?;
                    let nrows = to_u32(globals, &args[2])?;
                    let ncols = to_u32(globals, &args[3])?;
                    Ok(from_text_grid(try_(
                        globals,
                        ctx.new_text_grid(char_width, [nrows, ncols]),
                    )?))
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "text_grid_write",
                &["text_grid", "row", "col", "text"],
                None,
                |globals, args, _kwargs| {
                    let mut text_grid = to_text_grid_mut(globals, &args[0])?;
                    let row = to_u32(globals, &args[1])?;
                    let col = to_u32(globals, &args[2])?;
                    let text = Eval::expect_string(globals, &args[3])?;
                    text_grid.write_str([row, col], text);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::sdnew0(
                sr,
                "text_grid_rect_for_coord",
                &["text_grid", "row", "col"],
                None,
                |globals, args, _kwargs| {
                    let text_grid = to_text_grid(globals, &args[0])?;
                    let row = to_u32(globals, &args[1])?;
                    let col = to_u32(globals, &args[2])?;
                    let rect = text_grid.rect_for_coord([row, col]);
                    Ok(from_rect(rect))
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

fn unwr<T>(globals: &mut Globals, r: EvalResult<T>) -> T {
    match r {
        Ok(x) => x,
        Err(_) => {
            assert!(globals.print_if_error());
            panic!("Uncaught exception in game loop");
        }
    }
}

fn try_<T>(globals: &mut Globals, r: Result<T>) -> EvalResult<T> {
    match r {
        Ok(x) => Ok(x),
        Err(error) => globals.set_exc_str(&format!("{:?}", error)),
    }
}

fn from_rect(rect: Rect) -> Value {
    let [ul_x, ul_y] = rect.upper_left();
    let [lr_x, lr_y] = rect.lower_right();
    vec![
        Value::Float(ul_x as f64),
        Value::Float(ul_y as f64),
        Value::Float(lr_x as f64),
        Value::Float(lr_y as f64),
    ]
    .into()
}

fn to_rect(globals: &mut Globals, value: &Value) -> EvalResult<Rect> {
    let list = Eval::expect_list(globals, value)?;
    if list.len() == 2 {
        let (pt1, pt2) = Eval::unpack_pair(globals, value)?;
        let pt1 = to_2_f32(globals, &pt1)?;
        let pt2 = to_2_f32(globals, &pt2)?;
        Ok([pt1, pt2].into())
    } else {
        Ok(to_4_f32(globals, value)?.into())
    }
}

fn to_2_f32(globals: &mut Globals, value: &Value) -> EvalResult<[f32; 2]> {
    let (a, b) = Eval::unpack_pair(globals, value)?;
    let a = to_f32(globals, &a)?;
    let b = to_f32(globals, &b)?;
    Ok([a, b])
}

fn to_4_f32(globals: &mut Globals, value: &Value) -> EvalResult<[f32; 4]> {
    let (a, b, c, d) = Eval::unpack4(globals, value)?;
    let a = to_f32(globals, &a)?;
    let b = to_f32(globals, &b)?;
    let c = to_f32(globals, &c)?;
    let d = to_f32(globals, &d)?;
    Ok([a, b, c, d])
}

fn to_color(globals: &mut Globals, value: &Value) -> EvalResult<Color> {
    let list = Eval::expect_list(globals, value)?;
    let [r, g, b, a] = if list.len() == 3 {
        let r = to_f32(globals, &list[0])?;
        let g = to_f32(globals, &list[1])?;
        let b = to_f32(globals, &list[2])?;
        [r, g, b, 1.0]
    } else {
        let (r, g, b, a) = Eval::unpack4(globals, value)?;
        let r = to_f32(globals, &r)?;
        let g = to_f32(globals, &g)?;
        let b = to_f32(globals, &b)?;
        let a = to_f32(globals, &a)?;
        [r, g, b, a]
    };
    Ok([r, g, b, a].into())
}

fn to_f32(globals: &mut Globals, value: &Value) -> EvalResult<f32> {
    Ok(Eval::expect_floatlike(globals, value)? as f32)
}

fn to_u32(globals: &mut Globals, value: &Value) -> EvalResult<u32> {
    let i = Eval::expect_int(globals, value)?;
    let i = Eval::check_u32(globals, i)?;
    Ok(i)
}

fn from_batch(batch: SpriteBatch) -> Value {
    Opaque::new(batch).into()
}

fn to_batch<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, SpriteBatch>> {
    Eval::expect_opaque(globals, value)
}

fn to_batch_mut<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<RefMut<'a, SpriteBatch>> {
    Eval::expect_opaque_mut(globals, value)
}

fn from_sheet(sheet: Rc<SpriteSheet>) -> Value {
    Opaque::new(sheet).into()
}

fn to_sheet<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, Rc<SpriteSheet>>> {
    Eval::expect_opaque(globals, value)
}

fn from_text_grid(text_grid: TextGrid) -> Value {
    Opaque::new(text_grid).into()
}

fn is_text_grid(value: &Value) -> bool {
    match value {
        Value::Opaque(opaque) => opaque.borrow::<TextGrid>().is_some(),
        _ => false,
    }
}

fn to_text_grid<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, TextGrid>> {
    Eval::expect_opaque(globals, value)
}

fn to_text_grid_mut<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<RefMut<'a, TextGrid>> {
    Eval::expect_opaque_mut(globals, value)
}

fn to_actx<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, ACtx>> {
    Eval::expect_opaque(globals, value)
}

fn to_actx_mut<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<RefMut<'a, ACtx>> {
    Eval::expect_opaque_mut(globals, value)
}
