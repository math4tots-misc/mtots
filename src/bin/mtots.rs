extern crate mtots;
extern crate rustyline;

fn main() {
    let mut globals = mtots::Globals::new();
    mtots::add_standard_modules(&mut globals);
    let delegate = Delegate(rustyline::Editor::<()>::new());
    mtots::main(globals, Some(delegate));
}

struct Delegate(rustyline::Editor<()>);

impl mtots::ReplDelegate for Delegate {
    fn getline(&mut self, continuation: bool) -> Option<String> {
        let readline = self.0.readline(if continuation { ".. " } else { ">> " });
        match readline {
            Ok(line) => {
                self.0.add_history_entry(line.as_str());
                Some(line)
            }
            Err(rustyline::error::ReadlineError::Eof)
            | Err(rustyline::error::ReadlineError::Interrupted) => None,
            Err(error) => {
                eprintln!("Error readling line: {:?}", error);
                None
            }
        }
    }
}
