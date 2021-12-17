use iced_core::{alignment};

use crate::styles;
use crate::tmenu::MainAction;
use std::fmt::{Debug, Formatter};
use iced_native::widget::{Text, Container};
use iced_native::Length;

type Element<'a> = iced_native::Element<'a, MainAction, iced_wgpu::Renderer>;

#[derive(Clone)]
pub struct MenuItem {
    pub index: usize,
    pub text: String,
    pub mnemonic: Option<String>,
    pub value: Option<String>,
    pub state: ItemState,
    pub width: Option<f64>,
}

impl Debug for MenuItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: '{}' mn={:?} value={:?}",
            self.index, self.text, self.mnemonic, self.value
        )
    }
}

impl MenuItem {
    pub fn create(text: &str, index: usize) -> MenuItem {
        let (mnemonic, value, text) = if text.starts_with("(") {
            if let Some(mnemonic_end) = text.find(")").filter(|i| *i > 1) {
                let mn = text[1..mnemonic_end].to_string();
                let key_value = text[(mnemonic_end + 1)..].trim();
                let (value, text) = MenuItem::parse_key_value(text, key_value);

                let text = format!("({}) {}", mn, text);
                (Some(mn), value, text)
            } else {
                let (value, text) = MenuItem::parse_key_value(text, text);
                (None, value, text.to_string())
            }
        } else {
            let (value, text) = MenuItem::parse_key_value(text, text);
            (None, value, text.to_string())
        };

        MenuItem {
            index,
            text,
            mnemonic,
            value,
            state: ItemState::Visible,
            width: None,
        }
    }

    fn parse_key_value<'a>(all_text: &'a str, to_parse: &'a str) -> (Option<String>, &'a str) {
        if let Some(value_sep) = to_parse.find('=') {
            let value = &to_parse[0..value_sep];
            let text = &to_parse[value_sep + 1..];
            if !value.is_empty() && !text.trim().is_empty() {
                (Some(value.to_string()), text)
            } else {
                (None, all_text)
            }
        } else {
            if !to_parse.is_empty() {
                (None, to_parse)
            } else {
                (None, all_text)
            }
        }
    }

    pub fn value(&self) -> &str {
        self.value.as_deref().unwrap_or(self.text.as_str())
    }

    pub fn view<'a>(&self) -> Option<Element<'a>> {
        if self.state == ItemState::Hidden {
            return None;
        }
        let text = Text::new(self.text.clone()).vertical_alignment(alignment::Vertical::Center);
        let text = Container::new(text)
            .height(Length::Units(30))
            .align_y(alignment::Vertical::Center);
        let result = match self.state {
            ItemState::Active => text.style(styles::ActiveItem),
            _ => text.style(styles::DefaultItem),
        };
        Some(result.into())
    }

    pub fn visible(&self) -> bool {
        self.state != ItemState::Hidden
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItemState {
    Hidden,
    Visible,
    Active,
}

#[cfg(test)]
mod test {
    use crate::menu_item::MenuItem;

    #[test]
    fn parse_menu_item() {
        assert_item("a", "a", None, None, 0);
        assert_item("", "", None, None, 42);
        assert_item("a=b", "b", None, Some("a"), 0);
        assert_item("a=b=c", "b=c", None, Some("a"), 1);
        assert_item("a=", "a=", None, None, 2);
        assert_item("=b", "=b", None, None, 3);
        assert_item(" ", " ", None, None, 4);
        assert_item("(x) entry", "(x) entry", Some("x"), None, 3);
        assert_item("(x) a=b", "(x) b", Some("x"), Some("a"), 1);
        assert_item("(x)entry", "(x) entry", Some("x"), None, 2);
        assert_item("()entry", "()entry", None, None, 2);
        assert_item("(x)\t entry", "(x) entry", Some("x"), None, 2);
        assert_item(
            "(mn) value=the text",
            "(mn) the text",
            Some("mn"),
            Some("value"),
            42,
        );
    }

    fn assert_item(input: &str, text: &str, mn: Option<&str>, value: Option<&str>, index: usize) {
        let item = MenuItem::create(input, index);
        assert_eq!(item.text.as_str(), text);
        assert_eq!(item.mnemonic.as_deref(), mn);
        assert_eq!(item.value.as_deref(), value);
        assert_eq!(item.index, index);

        if let Some(value) = value {
            assert_eq!(item.value(), value);
        } else {
            assert_eq!(item.value(), text);
        }
    }
}
