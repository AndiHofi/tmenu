use crate::filter::{create_filter_factory, Filter, FilterFactory, Match};
use crate::menu_item::{ItemState, MenuItem};
use crate::styles;
use crate::tmenu_settings::TMenuSettings;
use iced::application::Update;
use iced::widget::{container, row, rule, text_input, Row};
use iced_core::keyboard::key::Named;
use iced_core::keyboard::{Event, Key, Modifiers};
use iced_core::{Length, Padding, Widget};
use iced_futures::{subscription, Subscription};
use iced_winit::runtime::{task, Action, Program, Task};
use std::cell::Cell;
use std::rc::Rc;
use iced::window;
use iced_core::widget::operation;
use iced_winit::runtime::task::widget;

type TElement<'a> = iced::Element<'a, MainAction>;

#[derive(Debug)]
pub struct TMenu {
    available_options: Vec<MenuItem>,
    auto_accept: bool,
    case_insensitive: bool,
    allow_undefined: bool,
    fuzzy: bool,
    text_changed: bool,
    verbose: bool,
    input: String,
    exit_state: Rc<Cell<ExitState>>,
    filter_factory: Box<dyn FilterFactory>,
}

impl TMenu {
    fn action_abort(&mut self) {
        self.exit_state.set(ExitState::Abort);
    }

    fn select_next(&mut self, offset: isize) {
        assert!(offset == -1 || offset == 1);
        let mut visible = all_visible(&mut self.available_options);
        let count = visible.len() as isize;
        if count > 0 {
            if let Some((last_active_index, last_active)) = visible
                .iter_mut()
                .map(|(_, i)| i)
                .enumerate()
                .find(|(_, i)| i.state == ItemState::Active)
            {
                last_active.state = ItemState::Visible;
                let mut next_active = last_active_index as isize + offset;
                if next_active >= count {
                    next_active -= count;
                } else if next_active < 0 {
                    next_active += count;
                }
                if let Some((_, i)) = visible.get_mut(next_active as usize) {
                    i.state = ItemState::Active;
                }
            } else if offset == 1 {
                visible.first_mut().unwrap().1.state = ItemState::Active;
            } else if offset == -1 {
                visible.last_mut().unwrap().1.state = ItemState::Active;
            }
        }
    }

    fn take_text(&mut self) {
        if let Some((_, item)) = find_active(&mut self.available_options) {
            self.input = item.text.to_string();
        }
    }
}

impl TMenu {
    pub fn update(&mut self, message: MainAction) -> Task<MainAction> {
        match message {
            MainAction::Focus => {}
            MainAction::Abort => {
                self.action_abort();
            },
            MainAction::Exit => {
                if let Some((_, result)) = find_active(&mut self.available_options) {
                    println!("{}", result.value());
                    self.exit_state.set(ExitState::Exit);
                } else if self.allow_undefined && !self.input.is_empty() {
                    println!("{}", self.input);
                    self.exit_state.set(ExitState::Exit);
                } else {
                    self.action_abort();
                }
            }
            MainAction::Next => self.select_next(1),
            MainAction::NextTab => {
                self.select_next(1);
                self.take_text();
            }
            MainAction::Previous => self.select_next(-1),
            MainAction::PreviousTab => {
                self.select_next(-1);
                self.take_text();
            }
            MainAction::TextChanged(new_input) => {
                let filter = self.filter_factory.create(&new_input);

                apply_filter(&mut self.available_options, filter, !self.allow_undefined);
                self.input = new_input;

                if self.auto_accept
                    && self
                        .available_options
                        .iter()
                        .filter(|e| e.visible())
                        .count()
                        == 1
                {
                    if let Some((_, result)) = find_active(&mut self.available_options) {
                        println!("{}", result.value());
                        self.exit_state.set(ExitState::Exit);
                    }
                }
            }
        };

        if matches!(self.exit_state.get(), ExitState::Continue) {
            task::effect(Action::widget(operation::focusable::focus(iced_core::widget::Id::new("input"))))
        } else {
            window::get_latest().and_then(window::close)
        }
    }

    pub fn view(&self) -> TElement<'_> {
        let main_input = text_input("option", &self.input)
            .id("input")
            .on_input(|input| MainAction::TextChanged(input))
            .padding(Padding {
                top: 5.0,
                right: 0.0,
                bottom: 5.0,
                left: 0.0,
            })
            .on_submit(MainAction::Exit);
        let mut children: Vec<TElement> = Vec::new();
        children.push(
            container(main_input)
                .width(Length::Fixed(300.0))
                .height(Length::Fill)
                .max_width(300)
                .padding(styles::TEXT_INPUT_PADDING).into()
        );

        let mut items = Vec::new();

        let start_pos = find_active(&self.available_options)
            .map(|a| a.0.max(2) - 2)
            .unwrap_or(0);

        let mut iter = self
            .available_options
            .iter()
            .skip(start_pos)
            .flat_map(MenuItem::view);

        if let Some(i) = iter.next() {
            items.push(i);
        }

        iter.fold(&mut items, |c, i| {
            c.push(rule::Rule::vertical(6).into());
            c.push(i);
            c
        });

        children.push(row(items).into());
        row(children).into()
    }
}

type Flags = TMenuSettings;

impl TMenu {
    pub fn new(flags: TMenuSettings) -> Self {
        let filter_factory = create_filter_factory(&flags);
        if flags.verbose {
            eprintln!("\n\n{:?}", filter_factory);
        }
        let mut app = TMenu {
            available_options: flags.available_options,
            auto_accept: flags.auto_accept,
            case_insensitive: flags.case_insensitive,
            allow_undefined: flags.allow_undefined,
            fuzzy: flags.fuzzy,
            text_changed: false,
            verbose: flags.verbose,
            input: String::new(),
            exit_state: flags.exit_state,
            filter_factory,
        };
        if !app.allow_undefined {
            if let Some(first) = app.available_options.first_mut() {
                first.state = ItemState::Active;
            }
        }
        app
    }

    fn title(&self) -> String {
        "tmenu".to_string()
    }

    pub fn subscription(&self) -> Subscription<MainAction> {
        iced::keyboard::on_key_press(on_key_pressed)
    }

    fn should_exit(&self) -> bool {
        if let ExitState::Continue = self.exit_state.get() {
            false
        } else {
            true
        }
    }
}

fn apply_filter<'a, 'b>(
    items: &'b mut [MenuItem],
    mut filter: Box<dyn Filter<'a> + 'a>,
    update_selection: bool,
) {
    let previous_active = find_active(items).map(|e| e.0);

    let mut match_offset = None;

    for i in items.iter_mut() {
        let result = filter.match_item(i);
        i.state = match result {
            Match::NoMatch => ItemState::Hidden,
            Match::Index(index) => {
                match_offset = Some(index);
                ItemState::Visible
            }
            Match::Match => ItemState::Visible,
        }
    }

    let mut all_visible = all_visible(items);

    let match_count = all_visible.len();
    if match_count == 0 {
        return;
    }

    if !update_selection {
        return;
    }

    select_item(previous_active, match_offset, &mut all_visible, match_count);
}

fn select_item(
    previous_active: Option<usize>,
    match_offset: Option<u32>,
    all_visible: &mut [(usize, &mut MenuItem)],
    match_count: usize,
) {
    let to_activate = if let Some(match_offset) = match_offset {
        if let Some((_, i)) = all_visible.get_mut(match_offset as usize % match_count) {
            i
        } else {
            all_visible.first_mut().map(|(_, i)| i).unwrap()
        }
    } else if let Some(previous_active) = previous_active {
        if let Some((_, i)) = all_visible.iter_mut().find(|(i, _)| *i == previous_active) {
            i
        } else {
            all_visible.first_mut().map(|(_, i)| i).unwrap()
        }
    } else {
        all_visible.first_mut().map(|(_, i)| i).unwrap()
    };

    to_activate.state = ItemState::Active;
}

fn all_visible(items: &mut [MenuItem]) -> Vec<(usize, &mut MenuItem)> {
    items
        .iter_mut()
        .enumerate()
        .filter(|(_, i)| i.state != ItemState::Hidden)
        .collect()
}

fn find_active(items: &[MenuItem]) -> Option<(usize, &MenuItem)> {
    items
        .iter()
        .enumerate()
        .find(|i| i.1.state == ItemState::Active)
}

fn on_key_pressed(key_code: Key, modifiers: Modifiers) -> Option<MainAction> {
    use MainAction::*;
    let action = match key_code {
        Key::Named(Named::Escape) => modifiers.is_empty().then(|| Abort).unwrap_or(Focus),
        Key::Named(Named::Tab) => {
            if modifiers.is_empty() {
                NextTab
            } else if modifiers == Modifiers::SHIFT {
                PreviousTab
            } else {
                Focus
            }
        }
        Key::Named(Named::ArrowRight) => {
            if modifiers.is_empty() {
                Next
            } else {
                Focus
            }
        }
        Key::Named(Named::ArrowLeft) => {
            if modifiers.is_empty() {
                Previous
            } else {
                Focus
            }
        }
        _ => Focus,
    };
    Some(action)
}

// fn global_keyboard_handler(
//     event: iced_native::Event,
//     status: iced_native::event::Status,
// ) -> Option<MainAction> {
//     fn on_key_pressed(
//         key_code: KeyCode,
//         modifiers: Modifiers,
//         _status: iced_native::event::Status,
//     ) -> Option<MainAction> {
//         use MainAction::*;
//         let action = match key_code {
//             KeyCode::Escape => modifiers.is_empty().then(|| Abort).unwrap_or(Focus),
//             KeyCode::Tab => {
//                 if modifiers.is_empty() {
//                     NextTab
//                 } else if modifiers == Modifiers::SHIFT {
//                     PreviousTab
//                 } else {
//                     Focus
//                 }
//             }
//             KeyCode::Right => {
//                 if modifiers.is_empty() {
//                     Next
//                 } else {
//                     Focus
//                 }
//             }
//             KeyCode::Left => {
//                 if modifiers.is_empty() {
//                     Previous
//                 } else {
//                     Focus
//                 }
//             }
//             _ => Focus,
//         };
//         Some(action)
//     }
//     match event {
//         iced_native::Event::Keyboard(Event::KeyPressed {
//             key_code,
//             modifiers,
//         }) => on_key_pressed(key_code, modifiers, status),
//
//         _ => Some(MainAction::Focus),
//     }
// }

#[derive(Clone, Debug)]
pub enum MainAction {
    Focus,
    Abort,
    Exit,
    Next,
    NextTab,
    Previous,
    PreviousTab,
    TextChanged(String),
}

impl Default for MainAction {
    fn default() -> Self {
        Self::Focus
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ExitState {
    Continue,
    Exit,
    Abort,
}
