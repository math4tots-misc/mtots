use crate::mtry;
use crate::ConvertValue;
use crate::Result;
use std::path::Path;

#[derive(Clone)]
pub struct Image(ggez::graphics::Image);

impl From<Image> for ggez::graphics::Image {
    fn from(image: Image) -> Self {
        image.0
    }
}

impl From<ggez::graphics::Image> for Image {
    fn from(image: ggez::graphics::Image) -> Self {
        Self(image)
    }
}

impl Image {
    pub fn get(&self) -> &ggez::graphics::Image {
        &self.0
    }
    pub fn from_file<P: AsRef<Path>>(ctx: &mut ggez::Context, p: P) -> Result<Image> {
        let bytes = std::fs::read(p)?;
        Self::from_bytes(ctx, &bytes)
    }
    pub fn from_bytes(ctx: &mut ggez::Context, bytes: &[u8]) -> Result<Image> {
        let im = mtry!(image::load_from_memory(bytes)).to_rgba();
        let (width, height) = im.dimensions();
        let im = mtry!(ggez::graphics::Image::from_rgba8(
            ctx,
            width as u16,
            height as u16,
            &im
        ));
        Ok(Self(im))
    }
}

impl ConvertValue for Image {}
