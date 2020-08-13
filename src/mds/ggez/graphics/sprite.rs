use crate::ConvertValue;

pub struct SpriteBatch(ggez::graphics::spritebatch::SpriteBatch);

impl SpriteBatch {
    pub fn new(sb: ggez::graphics::spritebatch::SpriteBatch) -> Self {
        Self(sb)
    }
    pub fn get_mut(&mut self) -> &mut ggez::graphics::spritebatch::SpriteBatch {
        &mut self.0
    }
    pub fn get(&self) -> &ggez::graphics::spritebatch::SpriteBatch {
        &self.0
    }
}

impl ConvertValue for SpriteBatch {}

pub struct SpriteIdx(ggez::graphics::spritebatch::SpriteIdx);

impl From<SpriteIdx> for ggez::graphics::spritebatch::SpriteIdx {
    fn from(index: SpriteIdx) -> Self {
        index.0
    }
}

impl From<ggez::graphics::spritebatch::SpriteIdx> for SpriteIdx {
    fn from(index: ggez::graphics::spritebatch::SpriteIdx) -> Self {
        Self(index)
    }
}
