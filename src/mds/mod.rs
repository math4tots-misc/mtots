use crate::Globals;

// mod ggez;
mod json;
// mod rand;
// mod regex;

pub fn add_standard_modules(globals: &mut Globals) {
    globals.add(json::new()).unwrap();
    // add(globals, rand::NAME, rand::load);
    // add(globals, regex::NAME, regex::load);
    // add(globals, ggez::NAME, ggez::load);
}
