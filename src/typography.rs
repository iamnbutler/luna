use gpui::{hsla, px, relative, DefiniteLength, Pixels, Styled};

pub static DEFAULT_FONT: &str = "Berkeley Mono";
pub static DEFAULT_LABEL_FONT_SIZE: f32 = 11.0;
pub static DEFAULT_FONT_SIZE: f32 = 13.0;
pub static DEFAULT_SINGLE_LINE_LINE_HEIGHT: Pixels = Pixels(15.0);
pub static DEFAULT_TEXT_LINE_SPACING: DefiniteLength = DefiniteLength::Fraction(1.24);

pub trait StyleTypographyExt: Styled + Sized {
    fn typography_style(self) -> Self {
        self.font_family(DEFAULT_FONT)
            .text_size(px(DEFAULT_FONT_SIZE))
            .line_height(DEFAULT_SINGLE_LINE_LINE_HEIGHT)
            .text_color(hsla(0.0, 1.0, 1.0, 1.0))
    }
    fn label_style(self) -> Self {
        self.font_family(DEFAULT_FONT)
            .text_size(px(DEFAULT_LABEL_FONT_SIZE))
            .line_height(DEFAULT_SINGLE_LINE_LINE_HEIGHT)
            .text_color(hsla(0.0, 1.0, 1.0, 1.0))
    }
}

impl<E: Styled> StyleTypographyExt for E {}
