use super::*;
use ggez::audio::SoundSource;

mod sound;
mod wav;

pub use sound::*;

pub const NAME: &str = "a.ggez.audio";

pub(in super::super) fn new() -> NativeModule {
    NativeModule::new(NAME, |m| {
        m.dep("a.bytes", None, &[]);
        m.class::<SoundData, _>("SoundData", |cls| {
            cls.sfunc("from_bytes", ["bytes"], "", |globals, args, _| {
                let mut args = args.into_iter();
                let bytes = args.next().unwrap().convert::<Vec<u8>>(globals)?;
                let data = SoundData::from_bytes(&bytes);
                Ok(globals.new_handle::<SoundData>(data)?.into())
            });
            cls.sfunc("from_samples", ["samples"], "", |globals, args, _| {
                let mut args = args.into_iter();
                let samples = <Vec<i16>>::try_from(args.next().unwrap())?;
                let data = SoundData::from_samples(&samples)?;
                Ok(globals.new_handle::<SoundData>(data)?.into())
            });
        });
        m.class::<Source, _>("Source", |cls| {
            cls.sfunc("from_data", ["data"], "", |globals, args, _| {
                let mut args = args.into_iter();
                let data = args.next().unwrap().convert::<SoundData>(globals)?;
                let ctx = getctx(globals)?;
                let source = Source::from_data(ctx, data)?;
                Ok(globals.new_handle::<Source>(source)?.into())
            });
            cls.ifunc("set_pitch", ["pitch"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let pitch = args.next().unwrap().f32()?;
                owner.borrow_mut().get_mut().set_pitch(pitch);
                Ok(Value::Nil)
            });
            cls.ifunc("set_repeat", ["repeat"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let repeat = args.next().unwrap().truthy();
                owner.borrow_mut().get_mut().set_repeat(repeat);
                Ok(Value::Nil)
            });
            cls.ifunc("repeat", [], "", |owner, _globals, _, _| {
                Ok(owner.borrow().get().repeat().into())
            });
            cls.ifunc("play", [], "", |owner, _globals, _, _| {
                mtry!(owner.borrow_mut().get_mut().play());
                Ok(Value::Nil)
            });
            cls.ifunc("pause", [], "", |owner, _globals, _, _| {
                owner.borrow().get().pause();
                Ok(Value::Nil)
            });
            cls.ifunc("resume", [], "", |owner, _globals, _, _| {
                owner.borrow().get().resume();
                Ok(Value::Nil)
            });
            cls.ifunc("set_volume", ["value"], "", |owner, _globals, args, _| {
                let mut args = args.into_iter();
                let value = args.next().unwrap().f32()?;
                owner.borrow_mut().get_mut().set_volume(value);
                Ok(Value::Nil)
            });
            cls.ifunc("volume", [], "", |owner, _globals, _, _| {
                Ok(owner.borrow().get().volume().into())
            });
        });
    })
}
