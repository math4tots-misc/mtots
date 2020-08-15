use crate::Globals;

#[cfg(feature = "gamekit")]
mod ggez;

#[cfg(feature = "basekit")]
mod json;

#[cfg(feature = "basekit")]
mod rand;

#[cfg(feature = "basekit")]
mod regex;

pub fn add_standard_modules(globals: &mut Globals) {
    #[cfg(feature = "basekit")]
    {
        globals.add(json::new()).unwrap();
        globals.add(rand::new()).unwrap();
        globals.add(regex::new()).unwrap();
    }

    #[cfg(feature = "gamekit")]
    {
        ggez::add(globals);
    }
}
