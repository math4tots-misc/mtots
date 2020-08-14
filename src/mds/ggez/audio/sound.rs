use crate::mtry;
use crate::ConvertValue;
use crate::Result;

#[derive(Clone)]
pub struct SoundData(ggez::audio::SoundData);

impl SoundData {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(ggez::audio::SoundData::from_bytes(data))
    }
    pub fn get(&self) -> &ggez::audio::SoundData {
        &self.0
    }
}

impl ConvertValue for SoundData {}

pub struct Source(ggez::audio::Source);

impl Source {
    pub fn from_data(ctx: &mut ggez::Context, data: SoundData) -> Result<Self> {
        Ok(Self(mtry!(ggez::audio::Source::from_data(
            ctx,
            data.0.clone()
        ))))
    }
    pub fn get(&self) -> &ggez::audio::Source {
        &self.0
    }
    pub fn get_mut(&mut self) -> &mut ggez::audio::Source {
        &mut self.0
    }
}
