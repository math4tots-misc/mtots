
#[derive(Default)]
pub struct DrawParam(ggez::graphics::DrawParam);

impl From<&DrawParam> for ggez::graphics::DrawParam {
    fn from(dp: &DrawParam) -> Self {
        dp.0
    }
}

impl From<&mut DrawParam> for ggez::graphics::DrawParam {
    fn from(dp: &mut DrawParam) -> Self {
        dp.0
    }
}

impl From<DrawParam> for ggez::graphics::DrawParam {
    fn from(dp: DrawParam) -> Self {
        dp.0
    }
}

impl DrawParam {
    pub fn get(&self) -> ggez::graphics::DrawParam {
        self.0
    }
    pub fn color<C: Into<ggez::graphics::Color>>(&mut self, color: C) -> &mut Self {
        self.0 = self.0.color(color.into());
        self
    }
    pub fn dest<P: Into<ggez::mint::Point2<f32>>>(&mut self, dest: P) -> &mut Self {
        self.0 = self.0.dest(dest);
        self
    }
    pub fn offset<P: Into<ggez::mint::Point2<f32>>>(&mut self, offset: P) -> &mut Self {
        self.0 = self.0.offset(offset);
        self
    }
    pub fn scale<P: Into<ggez::mint::Vector2<f32>>>(&mut self, scale: P) -> &mut Self {
        self.0 = self.0.scale(scale);
        self
    }
    pub fn rotation(&mut self, rotation: f32) -> &mut Self {
        self.0 = self.0.rotation(rotation);
        self
    }
}
