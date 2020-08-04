use super::*;

pub(super) fn as_text(globals: &mut Globals, value: Value) -> EvalResult<HCow<Text>> {
    if value.is_handle::<Text>() {
        Ok(Eval::expect_handle(globals, &value)?.into())
    } else {
        Ok(HCow::Owned(to_text(globals, value)?))
    }
}

fn to_text(globals: &mut Globals, value: Value) -> EvalResult<Text> {
    let mut frags = Vec::new();
    to_textfragments(globals, value, &mut frags)?;
    let mut text = Text::default();
    for frag in frags {
        text.add(frag);
    }
    Ok(text)
}

fn to_textfragments(
    globals: &mut Globals,
    value: Value,
    out: &mut Vec<TextFragment>,
) -> EvalResult<()> {
    match value {
        Value::List(list) => match Rc::try_unwrap(list) {
            Ok(list) => {
                for item in list {
                    to_textfragments(globals, item, out)?;
                }
            }
            Err(list) => {
                for item in list.iter() {
                    to_textfragments(globals, item.clone(), out)?;
                }
            }
        },
        _ => out.push(to_textfragment(globals, value)?),
    }
    Ok(())
}

fn to_textfragment(globals: &mut Globals, value: Value) -> EvalResult<TextFragment> {
    match value {
        Value::String(string) => Ok(TextFragment::new(string.unwrap_or_clone())),
        Value::Map(map) => {
            let mut map = map.to_string_keys(globals)?;
            let mut frag = TextFragment::default();
            if let Some(textval) = map.remove("text") {
                frag.text = Eval::expect_string(globals, &textval)?.as_ref().to_owned();
            }
            if let Some(colorval) = map.remove("color") {
                frag.color = Some(to_color(globals, &colorval)?);
            }
            if let Some(scaleval) = map.remove("scale") {
                let fontscale = Eval::expect_floatlike(globals, &scaleval)? as f32;
                frag.scale = Some(graphics::Scale::uniform(fontscale));
            }
            if let Some(fontval) = map.remove("font") {
                let font = as_font(globals, fontval)?.unwrap_or_clone();
                frag.font = Some(font);
            }
            if !map.is_empty() {
                let keys: Vec<_> = map.keys().collect();
                return globals.set_exc_str(&format!("Unused fragment attributes: {:?}", keys));
            }
            Ok(frag)
        }
        _ => globals.set_exc_str(&format!("Expected text fragment")),
    }
}

fn as_font(globals: &mut Globals, value: Value) -> EvalResult<HCow<Font>> {
    if value.is_handle::<Font>() {
        Ok(Eval::expect_handle(globals, &value)?.into())
    } else {
        Ok(HCow::Owned(to_font(globals, value)?))
    }
}

fn to_font(globals: &mut Globals, _value: Value) -> EvalResult<Font> {
    // TODO
    globals.set_exc_str(&format!("Expected a ggez Font"))
}

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
