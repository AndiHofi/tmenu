pub struct Entry {
    pub mnemonic: Option<Box<str>>,
    pub value: Box<str>,
    pub index: usize,
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: ({}) '{}'",
            self.index,
            self.mnemonic.as_deref().unwrap_or(""),
            self.value
        )
    }
}

pub trait EntryFilter {
    fn get_acc(&self) -> &[Entry];

    fn get_input(&self) -> &str;

    fn value_match(&self, entry: &Entry) -> bool;
}
