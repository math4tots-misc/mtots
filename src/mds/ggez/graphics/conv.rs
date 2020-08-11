use super::*;

pub struct Color(ggez::graphics::Color);

impl From<ggez::graphics::Color> for Color {
    fn from(c: ggez::graphics::Color) -> Self {
        Self(c)
    }
}

impl From<Color> for ggez::graphics::Color {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl TryFrom<Value> for Color {
    type Error = Error;
    fn try_from(value: Value) -> Result<Color> {
        match &value {
            Value::List(list) if list.borrow().len() == 3 => {
                let [r, g, b] = <[f32; 3]>::try_from(value)?;
                Ok(Color((r, g, b).into()))
            }
            _ => {
                let [r, g, b, a] = <[f32; 4]>::try_from(value)?;
                Ok(Color((r, g, b, a).into()))
            }
        }
    }
}

impl TryFrom<&Value> for Color {
    type Error = Error;
    fn try_from(value: &Value) -> Result<Color> {
        TryFrom::try_from(value.clone())
    }
}

#[derive(Clone)]
pub struct Text(ggez::graphics::Text);

impl Text {
    pub fn get(&self) -> &ggez::graphics::Text {
        &self.0
    }
}

impl ConvertValue for Text {
    fn convert(globals: &mut Globals, value: &Value) -> Result<Text> {
        let mut fragments = Vec::new();
        to_fragments(globals, value, &mut fragments)?;
        let mut text = ggez::graphics::Text::default();
        for fragment in fragments {
            text.add(fragment);
        }
        Ok(text.into())
    }
}

fn to_fragments(
    globals: &mut Globals,
    value: &Value,
    out: &mut Vec<ggez::graphics::TextFragment>,
) -> Result<()> {
    match value {
        Value::List(list) => {
            for x in list.borrow().iter() {
                to_fragments(globals, x, out)?;
            }
        }
        _ => out.push(value.clone().convert::<TextFragment>(globals)?.into()),
    }
    Ok(())
}

impl From<ggez::graphics::Text> for Text {
    fn from(x: ggez::graphics::Text) -> Self {
        Self(x)
    }
}

impl From<Text> for ggez::graphics::Text {
    fn from(x: Text) -> Self {
        x.0
    }
}

#[derive(Clone)]
pub struct TextFragment(ggez::graphics::TextFragment);

impl TextFragment {
    pub fn get(&self) -> &ggez::graphics::TextFragment {
        &self.0
    }
}

impl From<TextFragment> for ggez::graphics::TextFragment {
    fn from(x: TextFragment) -> Self {
        x.0
    }
}

impl From<ggez::graphics::TextFragment> for TextFragment {
    fn from(x: ggez::graphics::TextFragment) -> Self {
        Self(x)
    }
}

impl ConvertValue for TextFragment {
    fn convert(globals: &mut Globals, value: &Value) -> Result<TextFragment> {
        match value {
            Value::String(string) => Ok(TextFragment(ggez::graphics::TextFragment::new(
                string.str(),
            ))),
            Value::Map(map) => {
                let mut map = map.to_string_keys()?;
                let mut frag = ggez::graphics::TextFragment::default();
                if let Some(textval) = map.remove("text") {
                    frag.text = textval.into_string()?.unwrap_or_clone();
                }
                if let Some(colorval) = map.remove("color") {
                    frag.color = Some(Color::try_from(colorval)?.into());
                }
                if let Some(scaleval) = map.remove("scale") {
                    let fontscale = scaleval.f32()?;
                    frag.scale = Some(ggez::graphics::Scale::uniform(fontscale));
                }
                if let Some(fontval) = map.remove("font") {
                    let font = fontval.convert::<Font>(globals)?;
                    frag.font = Some(font.into());
                }
                if !map.is_empty() {
                    let keys: Vec<_> = map.keys().collect();
                    return Err(rterr!("Unused fragment attributes: {:?}", keys));
                }
                Ok(TextFragment::from(frag))
            }
            _ => Err(rterr!("Expected TextFragment")),
        }
    }
}

#[derive(Clone)]
pub struct Font(ggez::graphics::Font);

impl From<ggez::graphics::Font> for Font {
    fn from(font: ggez::graphics::Font) -> Self {
        Self(font)
    }
}

impl From<Font> for ggez::graphics::Font {
    fn from(font: Font) -> Self {
        font.0
    }
}

impl ConvertValue for Font {}
