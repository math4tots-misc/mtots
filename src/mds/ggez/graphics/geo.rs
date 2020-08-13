use super::*;

#[derive(Clone, Copy)]
pub struct DrawMode(ggez::graphics::DrawMode);

impl From<DrawMode> for ggez::graphics::DrawMode {
    fn from(x: DrawMode) -> Self {
        x.0
    }
}

impl From<ggez::graphics::DrawMode> for DrawMode {
    fn from(x: ggez::graphics::DrawMode) -> Self {
        Self(x)
    }
}

impl TryFrom<Value> for DrawMode {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        let string = value.into_string()?;
        let dm = match string.str() {
            "fill" => ggez::graphics::DrawMode::fill().into(),
            "stroke" => ggez::graphics::DrawMode::stroke(2.0).into(),
            _ => return Err(rterr!("Expected 'fill' or 'stroke', but got {:?}", string)),
        };
        Ok(dm)
    }
}

#[derive(Clone)]
pub struct Rect(ggez::graphics::Rect);

impl From<Rect> for ggez::graphics::Rect {
    fn from(x: Rect) -> Self {
        x.0
    }
}

impl From<ggez::graphics::Rect> for Rect {
    fn from(x: ggez::graphics::Rect) -> Self {
        Self(x)
    }
}

impl ConvertValue for Rect {
    fn convert(_globals: &mut Globals, value: &Value) -> Result<Self> {
        match value {
            Value::List(list) => {
                let len = list.borrow().len();
                if len == 2 {
                    let [[x1, y1], [x2, y2]] = <[[f32; 2]; 2]>::try_from(value.clone())?;
                    let x = fmin(x1, x2);
                    let y = fmin(y1, y2);
                    let w = fmax(x1, x2) - x;
                    let h = fmax(y1, y2) - y;
                    Ok(Self(ggez::graphics::Rect::new(x, y, w, h)))
                } else {
                    let [x, y, w, h] = <[f32; 4]>::try_from(value.clone())?;
                    Ok(Self(ggez::graphics::Rect::new(x, y, w, h)))
                }
            }
            _ => Err(rterr!(
                "Expected Rect (either [[x1, y1], [x2, y2]], or [x, y, w, h]), but got {:?}",
                value
            )),
        }
    }
}

fn fmin(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

fn fmax(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
