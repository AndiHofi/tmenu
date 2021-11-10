use iced::{alignment, Container, Element, Length, Text};

use crate::styles;
use crate::tmenu::MainAction;
use std::fmt::{Debug, Formatter};

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
        write!(f, "{}: '{}'", self.index, self.text)
    }
}

impl MenuItem {
    pub fn create(text: &str, index: usize) -> MenuItem {
        let (mnemonic, value, text) = if text.starts_with("(") {
            if let Some(mnemonic_end) = text.find(")") {
                let rmain = &text[(mnemonic_end + 1)..].trim();
                let mn = &text[1..mnemonic_end];
                if !rmain.is_empty() && !mn.is_empty() {
                    (
                        Some(mn.to_string()),
                        Some(rmain.to_string()),
                        format!("({}) {}", mn, rmain),
                    )
                } else {
                    (None, None, text.to_string())
                }
            } else {
                (None, None, text.to_string())
            }
        } else {
            (None, None, text.to_string())
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

    pub fn value(&self) -> &str {
        self.value.as_deref().unwrap_or(self.text.as_str())
    }

    pub fn view<'a>(&self) -> Option<Element<'a, MainAction>> {
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
