use std::{collections::HashMap, time::SystemTime};

#[derive(Default)]
pub struct Autocompleter {
    pub entries: Vec<String>,
}

impl Autocompleter {
    /// Adds a new entry to the autocompleter, trims the old entries
    /// and saves to disk.
    pub fn add_to_disk_and_cleanup(entry: &str) {
        let mut autocompleter = Self::from_saved_entries();
        autocompleter.trim_old();

        let now = SystemTime::now().format("%Y-%m-%d");
        autocompleter.entries.insert();
    }

    pub fn from_saved_entries() -> Self {
        let data_path = match dirs::data_local_dir() {
            Some(dir) => dir.join("mahou.autocomplete.dat"),
            None => return Self::default(),
        };
    }

    pub fn save_to_disk(&self) {

    }

    pub fn trim_old(&mut self) {

    }
}

/// Represents a set of autocompleter entries.
/// Could've been a trie, but this is simpler. Maybe one day it'll be a trie.
/// TODO:
struct EntrySet {

}

