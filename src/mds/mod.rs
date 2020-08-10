use crate::Globals;

// mod ggez;
mod json;
mod rand;
// mod regex;

pub fn add_standard_modules(globals: &mut Globals) {
    globals.add(json::new()).unwrap();
    globals.add(rand::new()).unwrap();
    // add(globals, regex::NAME, regex::load);
    // add(globals, ggez::NAME, ggez::load);
}
