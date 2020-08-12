use super::*;

pub struct Mesh(ggez::graphics::Mesh);

impl ConvertValue for Mesh {}

impl Mesh {
    pub fn get(&self) -> &ggez::graphics::Mesh {
        &self.0
    }
}

pub struct MeshBuilder(Option<ggez::graphics::MeshBuilder>);

impl MeshBuilder {
    pub fn new() -> Self {
        Self(Some(ggez::graphics::MeshBuilder::new()))
    }
    pub fn get_mut(&mut self) -> Result<&mut ggez::graphics::MeshBuilder> {
        match self.0.as_mut() {
            Some(mb) => Ok(mb),
            None => Err(rterr!("This MeshBuilder is already exhausted")),
        }
    }
    pub fn transfer(&mut self) -> Result<ggez::graphics::MeshBuilder> {
        match std::mem::replace(&mut self.0, None) {
            Some(mb) => Ok(mb),
            None => Err(rterr!("This MeshBuilder is already exhausted")),
        }
    }
    pub fn build(&mut self, ctx: &mut ggez::Context) -> Result<Mesh> {
        let builder = self.transfer()?;
        let mesh = mtry!(builder.build(ctx));
        Ok(Mesh(mesh))
    }
}

impl TryFrom<MeshBuilder> for ggez::graphics::MeshBuilder {
    type Error = Error;
    fn try_from(mut mb: MeshBuilder) -> Result<Self> {
        mb.transfer()
    }
}
