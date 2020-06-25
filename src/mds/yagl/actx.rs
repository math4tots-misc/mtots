use crate::anyhow::Result;
use crate::yagl::AppContext;
use crate::yagl::Color;
use crate::yagl::SpriteBatch;
use crate::yagl::SpriteSheet;
use crate::yagl::TextGrid;
use crate::Class;
use crate::Eval;
use crate::EvalResult;
use crate::Globals;
use crate::Opaque;
use crate::Value;
use std::rc::Rc;

/// Wrapper around yagl::AppContext
/// In order to pass values to mtots, the value has to be static
/// But AppContext objects have lifetimes that are only guaranteed
/// to the callback.
///
/// This wrapper is to consolidate all the logic for how the
/// context is used to ensure that it's not used in an unsafe manner.
///
/// Also, allowing a raw 'static AppContext to be passed to mtots
/// would mean the reference could be moved and copied without detection.
/// By doing this wrapping, we have a means of at least detecting
/// if the value was improperly moved.
pub(super) struct ACtx {
    ctx: &'static mut AppContext<'static>,
}

impl ACtx {
    pub(super) fn call_handler(
        globals: &mut Globals,
        handler: &Value,
        ctx_class: &Rc<Class>,
        ctx: &mut AppContext,
        mut other_args: Vec<Value>,
    ) -> EvalResult<Value> {
        let ctx: Self = Self {
            ctx: unsafe { std::mem::transmute(ctx) },
        };
        let opaque_rc = Rc::new(Opaque::new(ctx));
        let result = {
            let opaque_val = Value::Opaque(opaque_rc.clone());
            let ctx_val = Class::instantiate(ctx_class, globals, vec![opaque_val], None)?;
            other_args.insert(0, ctx_val);
            Eval::call(globals, handler, other_args)
        };
        // We must ensure that there are no more references to ctx
        // TODO: Check that we're doing enough to ensure safety
        match Rc::try_unwrap(opaque_rc) {
            Ok(_) => result,
            Err(_) => {
                // If we get here, it means that the AppContext reference
                // has leaked.
                // Even raising an error is not safe in this case,
                // since the error could be caught and the AppContext reference
                // could propagate
                panic!(concat!(
                    "yagl::AppContext reference has leaked! ",
                    "It is no longer safe to continue",
                ))
            }
        }
    }

    pub fn exit(&mut self) {
        self.ctx.exit();
    }

    pub fn scale(&self) -> [f32; 2] {
        self.ctx.scale()
    }

    pub fn new_sheet_from_color<C: Into<Color>>(&mut self, color: C) -> Result<Rc<SpriteSheet>> {
        self.ctx.new_sheet_from_color(color)
    }

    pub fn new_batch(&mut self, sheet: Rc<SpriteSheet>) -> Result<SpriteBatch> {
        self.ctx.new_batch(sheet)
    }

    pub fn new_text_grid(&mut self, char_width: f32, dim: [u32; 2]) -> Result<TextGrid> {
        self.ctx.new_text_grid(char_width, dim)
    }
}
