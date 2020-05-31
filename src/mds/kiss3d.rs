//! JSON bindings
use crate::kiss3d;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::Opaque;
use crate::RcStr;
use crate::Value;
use kiss3d::light::Light;
use kiss3d::nalgebra;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::Unit;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;

pub const NAME: &str = "_kiss3d";

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::simple0(sr, "new_window", &["name"], |globals, args, _kwargs| {
                use kiss3d::light::Light;
                use kiss3d::window::Window;
                let name = Eval::expect_string(globals, &args[0])?;
                let mut window = Window::new(name);
                window.set_light(Light::StickToCamera);
                let window: RefCell<Window> = RefCell::new(window);
                Ok(Opaque::new(window).into())
            }),
            NativeFunction::simple0(
                sr,
                "add_cube",
                &["window", "xyz"],
                |globals, args, _kwargs| {
                    let window: Ref<RefCell<Window>> = Eval::expect_opaque(globals, &args[0])?;
                    let mut window: RefMut<Window> = window.borrow_mut();
                    let (x, y, z) = get_xyz(globals, &args[1])?;
                    let cube: RefCell<SceneNode> = RefCell::new(window.add_cube(x, y, z));
                    Ok(Opaque::new(cube).into())
                },
            ),
            NativeFunction::simple0(
                sr,
                "append_translation",
                &["node", "xyz"],
                |globals, args, _kwargs| {
                    let node: Ref<RefCell<SceneNode>> = Eval::expect_opaque(globals, &args[0])?;
                    let mut node: RefMut<SceneNode> = node.borrow_mut();
                    let (x, y, z) = get_xyz(globals, &args[1])?;
                    node.append_translation(&Vector3::new(x, y, z).into());
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(
                sr,
                "append_rotation",
                &["node", "xyz", "r"],
                |globals, args, _kwargs| {
                    let node: Ref<RefCell<SceneNode>> = Eval::expect_opaque(globals, &args[0])?;
                    let mut node: RefMut<SceneNode> = node.borrow_mut();
                    let rot = get_rot(globals, &args[1], &args[2])?;
                    node.append_rotation(&rot);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(
                sr,
                "set_node_color",
                &["node", "rgb"],
                |globals, args, _kwargs| {
                    let node: Ref<RefCell<SceneNode>> = Eval::expect_opaque(globals, &args[0])?;
                    let mut node: RefMut<SceneNode> = node.borrow_mut();
                    let (r, g, b) = get_xyz(globals, &args[1])?;
                    node.set_color(r, g, b);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(
                sr,
                "start_event_loop",
                &["window", "delegate", "tick_step"],
                |globals, args, _kwargs| {
                    let window: Ref<RefCell<Window>> = Eval::expect_opaque(globals, &args[0])?;
                    let delegate = &args[1];
                    let on_tick = {
                        let key = globals.intern_str("tick");
                        Eval::getattr(globals, delegate, key)
                    };
                    let tick_step: f64 = Eval::expect_floatlike(globals, &args[2])?;
                    let mut last_tick = std::time::Instant::now();
                    let mut instants = VecDeque::<std::time::Instant>::new();
                    let font = kiss3d::text::Font::default();

                    while window.borrow_mut().render() {
                        let instant = std::time::Instant::now();
                        if let Some(tick_cb) = &on_tick {
                            let dur_f64 = (instant - last_tick).as_secs_f64();
                            if dur_f64 >= tick_step {
                                last_tick = instant;
                                Eval::call(globals, tick_cb, vec![dur_f64.into()])?;
                            }
                        }
                        while let Some(i) = instants.front() {
                            if (instant - *i).as_secs_f64() > 1.0 {
                                instants.pop_front();
                            } else {
                                break;
                            }
                        }
                        instants.push_back(instant);
                        window.borrow_mut().draw_text(
                            &format!("frames/sec = {}", instants.len()),
                            &nalgebra::Point2::origin(),
                            120.0,
                            &font,
                            &nalgebra::Point3::new(0.0, 1.0, 1.0),
                        );
                        let width = window.borrow().width();
                        let height = window.borrow().height();
                        window.borrow_mut().draw_text(
                            &format!("size = ({}, {})", width, height),
                            &nalgebra::Point2::new(0.0, 120.0),
                            120.0,
                            &font,
                            &nalgebra::Point3::new(0.0, 1.0, 1.0),
                        );
                        window.borrow_mut().draw_text(
                            &format!("middle row"),
                            &nalgebra::Point2::new(0.0, (height as f32) - 120.0),
                            120.0,
                            &font,
                            &nalgebra::Point3::new(0.0, 1.0, 1.0),
                        );
                        window.borrow_mut().draw_text(
                            &format!("bottom row"),
                            &nalgebra::Point2::new(0.0, 2.0 * (height as f32) - 120.0),
                            120.0,
                            &font,
                            &nalgebra::Point3::new(0.0, 1.0, 1.0),
                        );
                        for event in window.borrow().events().iter() {
                            println!("event -> {:?}", event.value);
                        }
                    }

                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(sr, "main", &[], |_globals, _args, _| {
                let mut window = Window::new("Kiss3d: cube");
                let mut c = window.add_cube(0.3, 0.3, 0.3);

                c.set_color(1.0, 0.0, 0.0);

                window.set_light(Light::StickToCamera);

                let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

                while window.render() {
                    for event in window.events().iter() {
                        println!("event -> {:?}", event.value);
                    }
                    c.prepend_to_local_rotation(&rot);
                }

                Ok(Value::Nil)
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

fn get_f32(globals: &mut Globals, x: &Value) -> EvalResult<f32> {
    Ok(Eval::expect_floatlike(globals, &x)? as f32)
}

fn get_xyz(globals: &mut Globals, xyz: &Value) -> EvalResult<(f32, f32, f32)> {
    let (x, y, z) = Eval::unpack_triple(globals, xyz)?;
    let x = get_f32(globals, &x)?;
    let y = get_f32(globals, &y)?;
    let z = get_f32(globals, &z)?;
    Ok((x, y, z))
}

fn get_rot(globals: &mut Globals, xyz: &Value, r: &Value) -> EvalResult<UnitQuaternion<f32>> {
    let (x, y, z) = get_xyz(globals, xyz)?;
    let r = get_f32(globals, r)?;
    let vec3 = Vector3::new(x, y, z);
    let unit = Unit::new_normalize(vec3);
    Ok(UnitQuaternion::from_axis_angle(&unit, r))
}
