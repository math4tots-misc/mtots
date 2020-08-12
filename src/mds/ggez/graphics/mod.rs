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
                let textval = args.next().unwrap();
                let text = textval.to_xref::<Text>(globals)?;
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
        m.func(
            "queue_text",
            ArgSpec::builder().req("text").def("x", 0).def("y", 0).def("color", ()),
            "",
            |globals, args, _| {
                let mut args = args.into_iter();
                let textval = args.next().unwrap();
                let text = textval.to_xref::<Text>(globals)?;
                let x = f32::try_from(args.next().unwrap())?;
                let y = f32::try_from(args.next().unwrap())?;
                let color: Option<ggez::graphics::Color> = match args.next().unwrap() {
                    Value::Nil => None,
                    value => Some(Color::try_from(value)?.into()),
                };
                let ctx = getctx(globals)?;
                ggez::graphics::queue_text(
                    ctx,
                    text.get(),
                    [x, y],
                    color,
                );
                Ok(Value::Nil)
            },
        );
        m.func(
            "draw_queued_text",
            ArgSpec::builder().def("x", 0).def("y", 0),
            "",
            |globals, args, _| {
                let mut args = args.into_iter();
                let x = f32::try_from(args.next().unwrap())?;
                let y = f32::try_from(args.next().unwrap())?;
                let ctx = getctx(globals)?;
                mtry!(ggez::graphics::draw_queued_text(
                    ctx,
                    DrawParam::default().dest([x, y]),
                    None,
                    ggez::graphics::FilterMode::Linear,
                ));
                Ok(Value::Nil)
            },
        );
        m.class::<Text, _>("Text", |cls| {
            cls.sfunc("__call", ["arg"], "", |globals, args, _| {
                let mut args = args.into_iter();
                let arg = args.next().unwrap();
                let text = arg.convert::<Text>(globals)?;
                Ok(globals.new_handle(text)?.into())
            });
            cls.ifunc("width", [], "", |owner, globals, _, _| {
                let ctx = getctx(globals)?;
                Ok(owner.borrow().get().width(ctx).into())
            });
            cls.ifunc("height", [], "", |owner, globals, _, _| {
                let ctx = getctx(globals)?;
                Ok(owner.borrow().get().height(ctx).into())
            });
            cls.ifunc("contents", [], "", |owner, _globals, _, _| {
                Ok(owner.borrow().get().contents().into())
            });
        });
    })
}
