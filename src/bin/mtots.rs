extern crate mtots;

fn main() {
    let mut globals = mtots::Globals::new();
    mtots::add_standard_modules(&mut globals);
    mtots::main(&mut globals);
}
