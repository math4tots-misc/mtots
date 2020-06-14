use crate::EvalResult;
use crate::Globals;
use crate::Symbol;
use crate::Value;
use std::collections::HashMap;

pub struct Context<'a> {
    globals: &'a mut Globals,
    scope_stack: Vec<Scope>,
    pos: usize,
    bytes: &'a [u8],
}

impl<'a> Context<'a> {
    pub(crate) fn new(globals: &'a mut Globals, bytes: &'a [u8]) -> Context<'a> {
        Context {
            globals,
            scope_stack: vec![Scope(HashMap::new())],
            pos: 0,
            bytes,
        }
    }
    pub fn err<R, S: Into<String>>(&mut self, s: S) -> EvalResult<R> {
        self.globals.set_exc_str(&s.into())
    }
    pub fn globals_mut_and_scope(&mut self) -> (&mut Globals, &Scope) {
        let globals = &mut self.globals;
        let scope = self.scope_stack.last().unwrap();
        (globals, scope)
    }
    pub fn scope_mut(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }
    pub fn peek(&mut self, n: usize) -> EvalResult<&'a [u8]> {
        match self.bytes.get(self.pos..self.pos + n) {
            Some(s) => Ok(s),
            None => self.err("Tried to peek beyond end"),
        }
    }
    pub fn read(&mut self, n: usize) -> EvalResult<&'a [u8]> {
        self.pos += n;
        match self.bytes.get(self.pos - n..self.pos) {
            Some(s) => Ok(s),
            None => self.err("Tried to read beyond end"),
        }
    }
    pub fn save(&self) -> usize {
        self.pos
    }
    pub fn restore(&mut self, pos: usize) {
        self.pos = pos;
    }
}

pub struct Scope(HashMap<Symbol, Value>);

impl Scope {
    pub fn get(&self, key: Symbol) -> Option<&Value> {
        self.0.get(&key)
    }
    pub fn get_or_error(&self, globals: &mut Globals, key: Symbol) -> EvalResult<&Value> {
        match self.get(key) {
            Some(val) => Ok(val),
            None => globals.set_exc_str(&format!("Key {:?} not found", key)),
        }
    }
    pub fn set(&mut self, key: Symbol, value: Value) {
        self.0.insert(key, value);
    }
    pub fn map(&self) -> &HashMap<Symbol, Value> {
        &self.0
    }
}
