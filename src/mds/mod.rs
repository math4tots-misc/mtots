use crate::Globals;

#[cfg(feature = "gamekit")]
mod ggez;

#[cfg(feature = "basekit")]
mod json;

#[cfg(feature = "basekit")]
mod rand;

#[cfg(feature = "basekit")]
mod regex;

pub fn add_standard_modules(_globals: &mut Globals) {
    #[cfg(feature = "basekit")]
    {
        _globals.add_native_module(json::new()).unwrap();
        _globals.add_native_module(rand::new()).unwrap();
        _globals.add_native_module(regex::new()).unwrap();
    }

    #[cfg(feature = "gamekit")]
    {
        ggez::add(_globals);
    }
}
