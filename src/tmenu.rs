use std::sync::atomic::{Ordering, AtomicI32};

use iced::keyboard::{Event, KeyCode, Modifiers};
use iced::{scrollable, text_input, Command, Container, Element, Length, Row, Rule, Scrollable, Subscription, TextInput, Application};

use crate::filter::{create_filter_factory, Filter, FilterFactory, Match};
use crate::menu_item::{ItemState, MenuItem};
use crate::styles;
use iced_core::Padding;
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct TMenuSettings {
    pub case_insensitive: bool,
    pub allow_undefined: bool,
    pub filter_by_prefix: bool,
    pub fuzzy: bool,
    pub verbose: bool,
    pub available_options: Vec<MenuItem>,
}

#[derive(Debug)]
pub struct TMenu {
    available_options: Vec<MenuItem>,
    case_insensitive: bool,
    allow_undefined: bool,
    fuzzy: bool,
    text_changed: bool,
    verbose: bool,
    input: String,
    exit_state: ExitState,
    filter_factory: Box<dyn FilterFactory>,
    exit_code: Arc<Box<AtomicI32>>,

    text_input: text_input::State,
    scrollable: scrollable::State,
}

impl TMenu {
    fn action_abort(&mut self) {
        self.exit_state = ExitState::Abort;
        self.exit_code.store(1, Ordering::SeqCst);
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
            }
        }
    }
}

impl Application for TMenu {
    type Executor = iced::executor::Default;
    type Message = MainAction;
    type Flags = TMenuSettings;

    fn new(flags: TMenuSettings) -> (Self, Command<Self::Message>) {
        let filter_factory = create_filter_factory(&flags);
        if flags.verbose {
            eprintln!("\n\n{:?}", filter_factory);
        }
        let mut app = TMenu {
            available_options: flags.available_options,
            case_insensitive: flags.case_insensitive,
            allow_undefined: flags.allow_undefined,
            fuzzy: flags.fuzzy,
            text_changed: false,
            verbose: flags.verbose,
            input: String::new(),
            exit_state: ExitState::Continue,
            filter_factory,
            text_input: text_input::State::focused(),
            scrollable: scrollable::State::default(),
            exit_code: Arc::new(Box::new(AtomicI32::new(0))),
        };
        if let Some(first) = app.available_options.first_mut() {
            first.state = ItemState::Active;
        }
        (app, Command::none())
    }

    fn title(&self) -> String {
        "tmenu".to_string()
    }

    fn update(&mut self, message: MainAction) -> Command<Self::Message> {
        self.text_input.focus();
        match message {
            MainAction::Focus => {}
            MainAction::Abort => self.action_abort(),
            MainAction::Exit => {
                if let Some((_, result)) = find_active(&mut self.available_options) {
                    println!("{}", result.value());
                    self.exit_state = ExitState::Exit;
                    self.exit_code.store(0, Ordering::SeqCst);
                } else {
                    self.action_abort()
                }
            }
            MainAction::Next => self.select_next(1),
            MainAction::Previous => self.select_next(-1),
            MainAction::TextChanged(new_input) => {
                let filter = self.filter_factory.create(&new_input);

                apply_filter(&mut self.available_options, filter);
                self.input = new_input;
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(global_keyboard_handler)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let main_input = TextInput::new(&mut self.text_input, "option", &self.input, |input| {
            MainAction::TextChanged(input)
        })
        .padding(Padding {
            top: 5,
            right: 0,
            bottom: 5,
            left: 0,
        })
        .on_submit(MainAction::Exit);

        let mut main_container = Row::new();
        main_container = main_container.push(
            Container::new(main_input)
                .width(Length::Units(300))
                .height(Length::Fill)
                .max_width(300)
                .padding(styles::TEXT_INPUT_PADDING),
        );
        let mut item_container = Row::new();

        let start_pos = find_active(&mut self.available_options)
            .map(|a| a.0.max(2) - 2)
            .unwrap_or(0);

        let mut iter = self
            .available_options
            .iter()
            .skip(start_pos)
            .flat_map(MenuItem::view);

        if let Some(i) = iter.next() {
            item_container = item_container.push(i);
        }

        item_container = iter.fold(item_container, |c, i| c.push(Rule::vertical(12)).push(i));
        let item_scroll = Scrollable::new(&mut self.scrollable)
            .width(Length::Fill)
            .scrollbar_width(0)
            .scroller_width(0)
            .push(item_container);

        main_container.push(item_scroll).into()
    }

    fn should_exit(&self) -> bool {
        if let ExitState::Continue = self.exit_state {
            false
        } else {
            true
        }
    }

    fn on_exit(&mut self) -> Option<Box<dyn FnOnce()>> {
        let exit_code = self.exit_code.clone();
        Some(Box::new(move || std::process::exit(exit_code.load(Ordering::SeqCst))))
    }

}

fn apply_filter<'a, 'b>(items: &'b mut [MenuItem], mut filter: Box<dyn Filter<'a> + 'a>) {
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

    let mut to_activate = if let Some(match_offset) = match_offset {
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

fn find_active(items: &mut [MenuItem]) -> Option<(usize, &MenuItem)> {
    items
        .iter()
        .enumerate()
        .find(|i| i.1.state == ItemState::Active)
}

fn global_keyboard_handler(
    event: iced_native::Event,
    status: iced_native::event::Status,
) -> Option<MainAction> {
    fn on_key_pressed(
        key_code: KeyCode,
        modifiers: Modifiers,
        _status: iced_native::event::Status,
    ) -> Option<MainAction> {
        use MainAction::*;
        let action = match key_code {
            KeyCode::Escape => modifiers.is_empty().then(|| Abort).unwrap_or(Focus),
            KeyCode::Tab => {
                if modifiers.is_empty() {
                    Next
                } else if modifiers == Modifiers::SHIFT {
                    Previous
                } else {
                    Focus
                }
            }
            KeyCode::Right => {
                if modifiers.is_empty() {
                    Next
                } else {
                    Focus
                }
            }
            KeyCode::Left => {
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
    match event {
        iced_native::Event::Keyboard(Event::KeyPressed {
            key_code,
            modifiers,
        }) => on_key_pressed(key_code, modifiers, status),

        _ => Some(MainAction::Focus),
    }
}

#[derive(Clone, Debug)]
pub enum MainAction {
    Focus,
    Abort,
    Exit,
    Next,
    Previous,
    TextChanged(String),
}

impl Default for MainAction {
    fn default() -> Self {
        Self::Focus
    }
}

#[derive(Debug)]
enum ExitState {
    Continue,
    Exit,
    Abort,
}
