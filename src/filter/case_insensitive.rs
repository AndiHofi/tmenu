use crate::filter::{Filter, FilterFactory};
use crate::filter::entry_filter::{Entry, EntryFilter};
use crate::menu_item::MenuItem;

#[derive(Debug)]
pub struct CIFactory {
    entries: Vec<Entry>,
    starts_width: bool,
}

impl CIFactory {
    pub fn create(items: &[MenuItem], starts_width: bool) -> Self {
        let entries = items
            .into_iter()
            .enumerate()
            .map(|(index, i)| Entry {
                mnemonic: i.mnemonic.as_deref().map(|i| i.into()),
                value: i
                    .value
                    .as_deref()
                    .map(|v| v)
                    .unwrap_or_else(|| i.text.as_str())
                    .to_lowercase()
                    .into(),
                index,
            })
            .collect();

        CIFactory {
            entries,
            starts_width,
        }
    }
}

impl FilterFactory for CIFactory {
    fn create_filter<'b, 'a: 'b>(&'a mut self, input: &'b str) -> Box<dyn Filter<'b> + 'b> {
        let lower_case = input.to_lowercase();
        if self.starts_width {
            Box::new(CaseInsensitiveStartsWithFilter {
                lower_case,
                input,
                acc: &self.entries,
                current: 0,
            })
        } else {
            Box::new(CaseInsensitiveContainsFilter {
                lower_case,
                input,
                acc: &self.entries,
                current: 0,
            })
        }
    }
}

#[derive(Debug)]
pub struct CaseInsensitiveStartsWithFilter<'a> {
    lower_case: String,
    input: &'a str,
    acc: &'a [Entry],
    current: usize,
}

impl<'a> EntryFilter for CaseInsensitiveStartsWithFilter<'a> {
    fn get_acc(&self) -> &[Entry] {
        self.acc
    }

    fn get_input(&self) -> &str {
        self.input
    }

    fn value_match(&self, entry: &Entry) -> bool {
        entry.value.starts_with(&self.lower_case)
    }
}

#[derive(Debug)]
pub struct CaseInsensitiveContainsFilter<'a> {
    lower_case: String,
    input: &'a str,
    acc: &'a [Entry],
    current: usize,
}

impl<'a> EntryFilter for CaseInsensitiveContainsFilter<'a> {
    fn get_acc(&self) -> &[Entry] {
        self.acc
    }

    fn get_input(&self) -> &str {
        self.input
    }

    fn value_match(&self, entry: &crate::filter::entry_filter::Entry) -> bool {
        entry.value.contains(&self.lower_case)
    }
}
