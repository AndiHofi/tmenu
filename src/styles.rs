use iced_style::container::Style;
use iced_core::Color;
use iced_core::Padding;
use iced_style::Background;

pub struct DefaultItem;

pub const TEXT_INPUT_PADDING: Padding = Padding {
    top: 0,
    right: 12,
    bottom: 0,
    left: 6,
};

impl iced_style::container::StyleSheet for DefaultItem {
    fn style(&self) -> Style {
        Style::default()
    }
}

pub struct ActiveItem;

impl iced_style::container::StyleSheet for ActiveItem {
    fn style(&self) -> Style {
        Style {
            background: Some(Background::Color(Color::from_rgb8(150, 150, 230))),
            ..Style::default()
        }
    }
}
