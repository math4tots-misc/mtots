use crate::Stashable;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use std::fmt;

pub type Result<T> = std::result::Result<T, SdlError>;

pub(super) struct State {
    sdl: Option<Sdl>,
    event_pump: Option<EventPump>,
    video: Option<VideoSubsystem>,
}

impl Stashable for State {}

impl Default for State {
    fn default() -> Self {
        Self {
            sdl: None,
            event_pump: None,
            video: None,
        }
    }
}

impl State {
    pub fn sdl(&mut self) -> Result<&mut Sdl> {
        if self.sdl.is_none() {
            self.sdl = Some(sdl2::init()?);
        }
        Ok(self.sdl.as_mut().unwrap())
    }
    pub fn event_pump(&mut self) -> Result<&mut EventPump> {
        if self.event_pump.is_none() {
            self.event_pump = Some(self.sdl()?.event_pump()?);
        }
        Ok(self.event_pump.as_mut().unwrap())
    }
    pub fn video(&mut self) -> Result<&mut VideoSubsystem> {
        if self.video.is_none() {
            self.video = Some(self.sdl()?.video()?);
        }
        Ok(self.video.as_mut().unwrap())
    }
    pub fn new_window(&mut self, title: &str, width: u32, height: u32) -> Result<Window> {
        Ok(self.video()?.window(title, width, height).build()?)
    }
}

#[derive(Debug)]
pub struct SdlError(String);

impl std::error::Error for SdlError {}

impl fmt::Display for SdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<String> for SdlError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<sdl2::video::WindowBuildError> for SdlError {
    fn from(e: sdl2::video::WindowBuildError) -> Self {
        Self(format!("{:?}", e))
    }
}

impl From<sdl2::IntegerOrSdlError> for SdlError {
    fn from(e: sdl2::IntegerOrSdlError) -> Self {
        Self(format!("{:?}", e))
    }
}
