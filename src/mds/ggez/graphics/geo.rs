use super::*;

#[derive(Clone, Copy)]
pub struct DrawMode(ggez::graphics::DrawMode);

impl From<DrawMode> for ggez::graphics::DrawMode {
    fn from(dm: DrawMode) -> Self {
        dm.0
    }
}

impl From<ggez::graphics::DrawMode> for DrawMode {
    fn from(dm: ggez::graphics::DrawMode) -> Self {
        Self(dm)
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
