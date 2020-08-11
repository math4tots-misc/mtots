use super::*;

mod conv;
pub use conv::*;

pub const NAME: &str = "a.ggez.graphics";

pub(in super::super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.func("clear", ["color"], "", |globals, args, _| {
            let mut args = args.into_iter();
            let color = Color::try_from(args.next().unwrap())?;
            let ctx = getctx(globals)?;
            ggez::graphics::clear(ctx, color.into());
            Ok(Value::Nil)
        });
        m.func(
            "print",
            ArgSpec::builder().req("text").def("x", 0).def("y", 0),
            "",
            |globals, args, _| {
                let mut args = args.into_iter();
                let text = args.next().unwrap().convert::<Text>(globals)?;
                let x = f32::try_from(args.next().unwrap())?;
                let y = f32::try_from(args.next().unwrap())?;
                let ctx = getctx(globals)?;
                mtry!(ggez::graphics::draw(
                    ctx,
                    text.get(),
                    DrawParam::default().dest([x, y])
                ));
                Ok(Value::Nil)
            },
        );
    })
}
