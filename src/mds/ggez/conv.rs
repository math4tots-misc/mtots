use super::*;

pub(super) fn to_color(globals: &mut Globals, val: &Value) -> EvalResult<Color> {
    let list = Eval::expect_list(globals, val)?;
    if list.len() == 3 {
        let [r, g, b] = to_3f32(globals, val)?;
        Ok([r, g, b, 1.0].into())
    } else {
        Ok(to_4f32(globals, val)?.into())
    }
}

pub(super) fn to_3f32(globals: &mut Globals, val: &Value) -> EvalResult<[f32; 3]> {
    let list = Eval::expect_list(globals, val)?;
    if list.len() != 3 {
        return globals.set_exc_str(&format!(
            "Expected 3 numbers, but got {} values",
            list.len()
        ));
    }
    let x1 = Eval::expect_floatlike(globals, &list[0])? as f32;
    let x2 = Eval::expect_floatlike(globals, &list[1])? as f32;
    let x3 = Eval::expect_floatlike(globals, &list[2])? as f32;
    Ok([x1, x2, x3])
}

pub(super) fn to_4f32(globals: &mut Globals, val: &Value) -> EvalResult<[f32; 4]> {
    let list = Eval::expect_list(globals, val)?;
    if list.len() != 3 {
        return globals.set_exc_str(&format!(
            "Expected 3 numbers, but got {} values",
            list.len()
        ));
    }
    let x1 = Eval::expect_floatlike(globals, &list[0])? as f32;
    let x2 = Eval::expect_floatlike(globals, &list[1])? as f32;
    let x3 = Eval::expect_floatlike(globals, &list[2])? as f32;
    let x4 = Eval::expect_floatlike(globals, &list[3])? as f32;
    Ok([x1, x2, x3, x4])
}
