use crate::Globals;

mod ggez;
mod json;
mod rand;
mod regex;

pub fn add_standard_modules(globals: &mut Globals) {
    globals.add(json::new()).unwrap();
    globals.add(rand::new()).unwrap();
    globals.add(regex::new()).unwrap();
    ggez::add(globals);
}
