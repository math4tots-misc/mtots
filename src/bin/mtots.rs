extern crate mtots;
use mtots::add_standard_modules;
use mtots::Globals;

fn main() {
    let mut globals = Globals::new();
    add_standard_modules(&mut globals);
    mtots::climain(globals);
}
