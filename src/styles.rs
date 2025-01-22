use iced::widget::{container, text};
use iced_core::{Color, Theme};
use iced_core::Padding;
use iced_core::widget::text::Catalog;

pub struct DefaultItem;

pub const TEXT_INPUT_PADDING: Padding = Padding {
    top: 0f32,
    right: 12f32,
    bottom: 0f32,
    left: 6f32,
};


// impl StyleSheet for DefaultItem {
//     fn style(&self) -> Style {
//         Style::default()
//     }
// }
//
// pub struct ActiveItem;
//
// impl ActiveItem {
//     pub fn container_style(_theme: &Theme) -> container::Style {
//         Theme::Light.palette().into()
//     }
//
//     pub fn text_style(theme: &Theme) -> text::Style {
//         theme.into()
//     }
// }
