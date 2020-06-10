//! three bindings
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::HMap;
use crate::NativeFunction;
use crate::RcStr;
use crate::Value;
use crate::three;
use crate::mint;
use crate::Opaque;
use three::Geometry;
use three::Window;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
            NativeFunction::simple0(sr, "new_geometry", &["vertices"], |globals, args, _| {
                let vertices = expect_vertices(globals, &args[0])?;
                let geometry = Geometry::with_vertices(vertices);
                Ok(Opaque::new(geometry).into())
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
