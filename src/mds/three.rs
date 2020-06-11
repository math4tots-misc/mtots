//! three bindings
use crate::mint;
use crate::three;
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
use three::template::InstancedGeometry;
use three::Color;
use three::Geometry;
use three::Object;
use three::Window;

pub const NAME: &str = "a._three";

type Point3 = mint::Point3<f32>;

pub(super) fn load(globals: &mut Globals) -> EvalResult<HMap<RcStr, Rc<RefCell<Value>>>> {
    let sr = globals.symbol_registry();
    let mut map = HashMap::<RcStr, Value>::new();

    map.extend(
        vec![
            NativeFunction::simple0(sr, "new_window", &["title"], |globals, args, _| {
                let title = Eval::expect_string(globals, &args[0])?;
                let window = Window::new(title.str());
                Ok(Opaque::new(window).into())
            }),
            NativeFunction::simple0(
                sr,
                "set_background_color",
                &["window", "color"],
                |globals, args, _| {
                    let mut window = expect_window_mut(globals, &args[0])?;
                    let color = expect_color(globals, &args[1])?;
                    window.scene.background = three::Background::Color(color);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_orthographic_camera",
                &["window", "center", "extent_y", "range"],
                |globals, args, _| {
                    let mut window = expect_window_mut(globals, &args[0])?;
                    let (center_x, center_y) = expect_f32_pair(globals, &args[1])?;
                    let extent_y = Eval::expect_floatlike(globals, &args[2])? as f32;
                    let (range_start, range_end) = expect_f32_pair(globals, &args[3])?;
                    let camera = window.factory.orthographic_camera(
                        [center_x, center_y],
                        extent_y,
                        range_start..range_end,
                    );
                    from_camera(globals, camera)
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_perspective_camera",
                &["window", "fov_y", "range"],
                |globals, args, _| {
                    let mut window = expect_window_mut(globals, &args[0])?;
                    let fov_y = Eval::expect_floatlike(globals, &args[1])? as f32;
                    let range = expect_range(globals, &args[2])?;
                    let camera = window.factory.perspective_camera(fov_y, range);
                    from_camera(globals, camera)
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_geometry_from_vertices",
                &["vertices"],
                |globals, args, _| {
                    let vertices = expect_vertices(globals, &args[0])?;
                    let geometry = Geometry::with_vertices(vertices);
                    Ok(Opaque::new(geometry).into())
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_geometry_cuboid",
                &["width", "height", "depth"],
                |globals, args, _| {
                    let width = Eval::expect_floatlike(globals, &args[0])? as f32;
                    let height = Eval::expect_floatlike(globals, &args[1])? as f32;
                    let depth = Eval::expect_floatlike(globals, &args[2])? as f32;
                    let geometry = Geometry::cuboid(width, height, depth);
                    Ok(Opaque::new(geometry).into())
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_geometry_cylinder",
                &["radius_top", "radius_bottom", "height", "radius_segments"],
                |globals, args, _| {
                    let radius_top = Eval::expect_floatlike(globals, &args[0])? as f32;
                    let radius_bottom = Eval::expect_floatlike(globals, &args[1])? as f32;
                    let height = Eval::expect_floatlike(globals, &args[2])? as f32;
                    let radius_segments = Eval::expect_usize(globals, &args[3])?;
                    let geometry =
                        Geometry::cylinder(radius_top, radius_bottom, height, radius_segments);
                    Ok(Opaque::new(geometry).into())
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_geometry_uv_sphere",
                &["radius", "equatorial_segments", "meridional_segments"],
                |globals, args, _| {
                    let radius = Eval::expect_floatlike(globals, &args[0])? as f32;
                    let equatorial_segments = Eval::expect_usize(globals, &args[1])?;
                    let meridional_segments = Eval::expect_usize(globals, &args[2])?;
                    let geometry =
                        Geometry::uv_sphere(radius, equatorial_segments, meridional_segments);
                    Ok(Opaque::new(geometry).into())
                },
            ),
            NativeFunction::simple0(
                sr,
                "new_instanced_geometry",
                &["window", "geometry"],
                |globals, args, _| {
                    let mut window = expect_window_mut(globals, &args[0])?;
                    let geometry = move_geometry(globals, &args[1])?;
                    let instanced_geometry = window.factory.upload_geometry(geometry);
                    Ok(Opaque::new(instanced_geometry).into())
                },
            ),
            NativeFunction::simple0(sr, "new_material_basic", &["color"], |globals, args, _| {
                let color = expect_color(globals, &args[0])?;
                let material: three::Material = three::material::Basic {
                    color,
                    ..Default::default()
                }
                .into();
                Ok(Opaque::new(material).into())
            }),
            NativeFunction::simple0(
                sr,
                "new_mesh",
                &["window", "instanced_geometry", "material"],
                |globals, args, _| {
                    let mut window = expect_window_mut(globals, &args[0])?;
                    let instanced_geometry = expect_instanced_geometry(globals, &args[1])?;
                    let material = move_material(globals, &args[2])?;
                    let mesh = window
                        .factory
                        .create_instanced_mesh(&instanced_geometry, material);
                    from_mesh(globals, mesh)
                },
            ),
            NativeFunction::simple0(sr, "window_add", &["window", "obj"], |globals, args, _| {
                let mut window = expect_window_mut(globals, &args[0])?;
                let obj = expect_obj(globals, &args[1])?;
                obj.add_to(&mut window);
                Ok(Value::Nil)
            }),
            NativeFunction::simple0(
                sr,
                "main",
                &["window", "camera_cell", "update"],
                |globals, args, _| {
                    let window_val = &args[0];
                    let camera_cell = Eval::expect_cell(globals, &args[1])?;
                    let update = &args[2];
                    while expect_window_mut(globals, window_val)?.update() {
                        Eval::call(globals, update, vec![])?;
                        let camera_val = camera_cell.borrow();
                        let camera = expect_camera(globals, &camera_val)?;
                        expect_window_mut(globals, window_val)?.render(&camera);
                    }
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(
                sr,
                "set_transform",
                &["obj", "pos", "rot", "scale"],
                |globals, args, _| {
                    let obj = expect_obj(globals, &args[0])?;
                    let pos = expect_point3(globals, &args[1])?;
                    let rot = expect_rot(globals, &args[2])?;
                    let scale = Eval::expect_floatlike(globals, &args[3])? as f32;
                    obj.set_transform(pos, rot, scale);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(sr, "set_position", &["obj", "pos"], |globals, args, _| {
                let obj = expect_obj(globals, &args[0])?;
                let pos = expect_point3(globals, &args[1])?;
                obj.set_position(pos);
                Ok(Value::Nil)
            }),
            NativeFunction::simple0(
                sr,
                "set_orientation",
                &["obj", "rot"],
                |globals, args, _| {
                    let obj = expect_obj(globals, &args[0])?;
                    let rot = expect_rot(globals, &args[1])?;
                    obj.set_orientation(rot);
                    Ok(Value::Nil)
                },
            ),
            NativeFunction::simple0(sr, "set_scale", &["obj", "scale"], |globals, args, _| {
                let obj = expect_obj(globals, &args[0])?;
                let scale = Eval::expect_floatlike(globals, &args[1])? as f32;
                obj.set_scale(scale);
                Ok(Value::Nil)
            }),
            NativeFunction::simple0(
                sr,
                "look_at",
                &["obj", "eye", "target", "up"],
                |globals, args, _| {
                    let obj = expect_obj(globals, &args[0])?;
                    let eye = expect_point3(globals, &args[1])?;
                    let target = expect_point3(globals, &args[2])?;
                    let up = if let Value::Nil = &args[3] {
                        None
                    } else {
                        let (a, b, c) = expect_f32_triple(globals, &args[3])?;
                        Some([a, b, c].into())
                    };
                    obj.look_at(eye, target, up);
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

fn expect_point3(globals: &mut Globals, value: &Value) -> EvalResult<Point3> {
    let (a, b, c) = Eval::unpack_triple(globals, value)?;
    let a = Eval::expect_floatlike(globals, &a)? as f32;
    let b = Eval::expect_floatlike(globals, &b)? as f32;
    let c = Eval::expect_floatlike(globals, &c)? as f32;
    Ok([a, b, c].into())
}

fn expect_vertices(globals: &mut Globals, value: &Value) -> EvalResult<Vec<Point3>> {
    let iterator = Eval::iter(globals, value)?;
    let mut points = Vec::new();
    while let Some(value) = Eval::next(globals, &iterator)? {
        let point = expect_point3(globals, &value)?;
        points.push(point);
    }
    Ok(points)
}

fn expect_color(globals: &mut Globals, value: &Value) -> EvalResult<Color> {
    let i = Eval::expect_int(globals, value)?;
    Ok(i as Color)
}

fn move_geometry(globals: &mut Globals, value: &Value) -> EvalResult<Geometry> {
    Eval::move_opaque(globals, value)
}

fn expect_instanced_geometry<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<Ref<'a, InstancedGeometry>> {
    Eval::expect_opaque(globals, value)
}

fn move_material(globals: &mut Globals, value: &Value) -> EvalResult<three::Material> {
    Eval::move_opaque(globals, value)
}

fn expect_window_mut<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<RefMut<'a, Window>> {
    Eval::expect_opaque_mut(globals, value)
}

fn from_mesh(_: &mut Globals, mesh: three::Mesh) -> EvalResult<Value> {
    Ok(Opaque::new(Obj::Mesh(mesh)).into())
}

fn from_camera(_: &mut Globals, camera: three::camera::Camera) -> EvalResult<Value> {
    Ok(Opaque::new(Obj::Camera(camera)).into())
}

fn expect_camera<'a>(
    globals: &mut Globals,
    value: &'a Value,
) -> EvalResult<Ref<'a, three::camera::Camera>> {
    let obj = expect_obj(globals, value)?;
    if obj.camera().is_some() {
        Ok(Ref::map(obj, |obj| obj.camera().unwrap()))
    } else {
        globals.set_exc_str("Expected Camera, but got wrong Obj type")
    }
}

fn expect_obj<'a>(globals: &mut Globals, value: &'a Value) -> EvalResult<Ref<'a, Obj>> {
    Eval::expect_opaque(globals, value)
}

fn expect_f32_pair(globals: &mut Globals, value: &Value) -> EvalResult<(f32, f32)> {
    let (a, b) = Eval::unpack_pair(globals, value)?;
    let a = Eval::expect_floatlike(globals, &a)? as f32;
    let b = Eval::expect_floatlike(globals, &b)? as f32;
    Ok((a, b))
}

fn expect_f32_triple(globals: &mut Globals, value: &Value) -> EvalResult<(f32, f32, f32)> {
    let (a, b, c) = Eval::unpack_triple(globals, value)?;
    let a = Eval::expect_floatlike(globals, &a)? as f32;
    let b = Eval::expect_floatlike(globals, &b)? as f32;
    let c = Eval::expect_floatlike(globals, &c)? as f32;
    Ok((a, b, c))
}

fn expect_rot(globals: &mut Globals, value: &Value) -> EvalResult<mint::Quaternion<f32>> {
    let (a, b, c, d) = Eval::unpack4(globals, value)?;
    let a = Eval::expect_floatlike(globals, &a)? as f32;
    let b = Eval::expect_floatlike(globals, &b)? as f32;
    let c = Eval::expect_floatlike(globals, &c)? as f32;
    let d = Eval::expect_floatlike(globals, &d)? as f32;
    Ok(quaternion_from_floats(a, b, c, d))
}

/// Generate the proper quaternion given the axis of rotation
/// and amount to rotate by (angle is taken in radians)
fn quaternion_from_floats(ux: f32, uy: f32, uz: f32, angle: f32) -> mint::Quaternion<f32> {
    let scale = (ux * ux + uy * uy + uz * uz).sqrt();
    let angle2 = angle / 2.0;
    let scalar = angle2.cos();
    let factor = angle2.sin() / scale;
    [ux * factor, uy * factor, uz * factor, scalar].into()
}

fn expect_range(globals: &mut Globals, value: &Value) -> EvalResult<three::camera::ZRange> {
    let (a, b) = Eval::unpack_pair(globals, value)?;
    let a = Eval::expect_floatlike(globals, &a)? as f32;
    if let Value::Nil = b {
        Ok((a..).into())
    } else {
        let b = Eval::expect_floatlike(globals, &b)? as f32;
        Ok((a..b).into())
    }
}

/// Models three's 'Object' trait
enum Obj {
    Camera(three::camera::Camera),
    Mesh(three::Mesh),
}

impl Obj {
    fn camera(&self) -> Option<&three::camera::Camera> {
        if let Obj::Camera(c) = self {
            Some(c)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn mesh(&self) -> Option<&three::Mesh> {
        if let Obj::Mesh(m) = self {
            Some(m)
        } else {
            None
        }
    }

    fn add_to(&self, window: &mut Window) {
        match self {
            Obj::Camera(c) => window.scene.add(c),
            Obj::Mesh(c) => window.scene.add(c),
        }
    }

    fn look_at<E, T>(&self, eye: E, target: T, up: Option<mint::Vector3<f32>>)
    where
        E: Into<Point3>,
        T: Into<Point3>,
    {
        match self {
            Obj::Camera(c) => c.look_at(eye, target, up),
            Obj::Mesh(c) => c.look_at(eye, target, up),
        }
    }

    fn set_transform<P, Q>(&self, pos: P, rot: Q, scale: f32)
    where
        P: Into<Point3>,
        Q: Into<mint::Quaternion<f32>>,
    {
        match self {
            Obj::Camera(c) => c.set_transform(pos, rot, scale),
            Obj::Mesh(c) => c.set_transform(pos, rot, scale),
        }
    }

    fn set_position<P>(&self, pos: P)
    where
        P: Into<Point3>,
    {
        match self {
            Obj::Camera(c) => c.set_position(pos),
            Obj::Mesh(c) => c.set_position(pos),
        }
    }

    fn set_orientation<Q>(&self, rot: Q)
    where
        Q: Into<mint::Quaternion<f32>>,
    {
        match self {
            Obj::Camera(c) => c.set_orientation(rot),
            Obj::Mesh(c) => c.set_orientation(rot),
        }
    }

    fn set_scale(&self, scale: f32) {
        match self {
            Obj::Camera(c) => c.set_scale(scale),
            Obj::Mesh(c) => c.set_scale(scale),
        }
    }
}
