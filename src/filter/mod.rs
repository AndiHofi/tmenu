use std::fmt::Debug;

use entry_filter::EntryFilter;

use crate::filter::case_insensitive::CIFactory;
use crate::filter::case_sensitive::CSFactory;
use crate::filter::Match::NoMatch;
use crate::menu_item::MenuItem;
use crate::tmenu_settings::TMenuSettings;

mod case_insensitive;
mod case_sensitive;
mod entry_filter;

#[derive(PartialEq, Debug)]
pub enum Match {
    NoMatch,
    Match,
    Index(u32),
}

impl Match {
    fn or_else<M: Into<Match>, F: FnOnce() -> M>(self, alternative: F) -> Match {
        if let Self::NoMatch = self {
            alternative().into()
        } else {
            self
        }
    }
}

impl From<bool> for Match {
    fn from(v: bool) -> Self {
        if v {
            Match::Match
        } else {
            Match::NoMatch
        }
    }
}

pub fn create_filter_factory(settings: &TMenuSettings) -> Box<dyn FilterFactory> {
    if settings.case_insensitive {
        Box::new(CIFactory::create(
            &settings.available_options,
            settings.filter_by_prefix,
        ))
    } else {
        Box::new(CSFactory::create(settings))
    }
}

pub trait FilterFactory: Debug {
    fn create<'b, 'a: 'b>(&'a mut self, input: &'b str) -> Box<dyn Filter<'b> + 'b> {
        if input.is_empty() {
            Box::new(MatchAllFilter)
        } else {
            self.create_filter(input)
        }
    }

    fn create_filter<'b, 'a: 'b>(&'a mut self, input: &'b str) -> Box<dyn Filter<'b> + 'b>;
}

pub trait Filter<'a>: Debug {
    fn match_item(&mut self, item: &MenuItem) -> Match;
}

impl<'a, E: Debug + EntryFilter> Filter<'a> for E {
    fn match_item(&mut self, item: &MenuItem) -> Match {
        let entry = self.get_acc().get(item.index);
        entry
            .map(|e| {
                match_mnemonic_opt(e.mnemonic.as_deref(), self.get_input())
                    .or_else(|| self.value_match(e))
            })
            .unwrap_or(Match::NoMatch)
    }
}

#[derive(Debug)]
pub struct MatchAllFilter;

impl<'a> Filter<'a> for MatchAllFilter {
    fn match_item(&mut self, _item: &MenuItem) -> Match {
        Match::Match
    }
}

fn match_mnemonic_opt(mnemonic: Option<&str>, input: &str) -> Match {
    if let Some(mnemonic) = mnemonic {
        match_mnemonic(mnemonic, input)
    } else {
        NoMatch
    }
}

fn match_mnemonic(mnemonic: &str, input: &str) -> Match {
    let mn_len = mnemonic.chars().count();
    if mn_len >= input.chars().count() && mnemonic.starts_with(input) {
        Match::Match
    } else if mn_len == 1 {
        if input.starts_with(mnemonic) {
            Match::Index(input.chars().count() as u32)
        } else {
            Match::NoMatch
        }
    } else {
        Match::NoMatch
    }
}
