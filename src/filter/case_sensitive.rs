use crate::filter::{
    Filter, FilterFactory,
};
use crate::filter::entry_filter::{Entry, EntryFilter};
use crate::tmenu::TMenuSettings;

#[derive(Debug)]
pub struct CSFactory {
    acc: Vec<Entry>,
    starts_with: bool,
}

impl CSFactory {
    pub fn create(settings: &TMenuSettings) -> Self {
        let acc = settings
            .available_options
            .iter()
            .enumerate()
            .map(|(index, i)| Entry {
                mnemonic: i.mnemonic.as_deref().map(|s| s.into()),
                value: i.value.as_deref().unwrap_or(i.text.as_str()).into(),
                index,
            })
            .collect();
        CSFactory {
            acc,
            starts_with: settings.filter_by_prefix,
        }
    }
}

impl FilterFactory for CSFactory {
    fn create_filter<'b, 'a: 'b>(&'a mut self, input: &'b str) -> Box<dyn Filter<'b> + 'b> {
        if self.starts_with {
            Box::new(CaseSensitiveStartsWithFilter {
                input,
                acc: &self.acc,
            })
        } else {
            Box::new(CaseSensitiveContainsFilter {
                input,
                acc: &self.acc,
            })
        }
    }
}

#[derive(Debug)]
pub struct CaseSensitiveStartsWithFilter<'a> {
    input: &'a str,
    acc: &'a [Entry],
}

impl<'a> EntryFilter for CaseSensitiveStartsWithFilter<'a> {
    fn get_acc(&self) -> &[Entry] {
        self.acc
    }

    fn get_input(&self) -> &str {
        self.input
    }

    fn value_match(&self, entry: &Entry) -> bool {
        entry.value.starts_with(self.input)
    }
}

#[derive(Debug)]
pub struct CaseSensitiveContainsFilter<'a> {
    input: &'a str,
    acc: &'a [Entry],
}

impl<'a> EntryFilter for CaseSensitiveContainsFilter<'a> {
    fn get_acc(&self) -> &[Entry] {
        self.acc
    }

    fn get_input(&self) -> &str {
        self.input
    }

    fn value_match(&self, entry: &Entry) -> bool {
        entry.value.contains(self.input)
    }
}
