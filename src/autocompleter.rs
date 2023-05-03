use inquire::autocompletion::Replacement;
use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    time::{Duration, SystemTime},
};

fn data_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("mahou.autocomplete.dat"))
}

#[derive(Default, Clone)]
pub struct Autocompleter {
    pub entries: Vec<String>,
}

impl Autocompleter {
    pub fn from_saved_entries() -> Self {
        let entries = match EntrySet::from_disk() {
            Some(mut entryset) => std::mem::take(&mut entryset.entries),
            None => return Self::default(),
        };

        let entries = entries.into_iter().map(|e| e.string).collect();
        Self { entries }
    }
}

impl inquire::Autocomplete for Autocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let input = input.to_lowercase();
        Ok(self
            .entries
            .iter()
            .filter(|e| e.to_lowercase().contains(&input))
            .map(|e| e.to_owned())
            .collect())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        Ok(highlighted_suggestion)
    }
}

/// An autocompleter entry
#[derive(Debug)]
pub struct Entry {
    pub string: String,
    pub last_used: SystemTime,
}

/// Represents a set of autocompleter entries that are persisted on disk.
/// TODO: Could've been a trie, but this is simpler. Maybe one day it'll be a trie.
#[derive(Debug)]
pub struct EntrySet {
    entries: Vec<Entry>,
    saved: bool,
}

impl Default for EntrySet {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            saved: true,
        }
    }
}

impl EntrySet {
    pub fn from_disk() -> Option<Self> {
        let file = match fs::File::open(data_path()?) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Some(Self::default()),
            _ => return None,
        };
        let reader = BufReader::new(file);

        let entries = reader
            .lines()
            .map(|line| {
                let line = line.unwrap();
                let mut tokens = line.splitn(2, ' ');
                let last_used = tokens.next().unwrap().parse::<u64>().unwrap();
                let last_used = SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_secs(last_used))
                    .unwrap();
                let entry = tokens.next().unwrap();
                Entry {
                    string: entry.to_owned(),
                    last_used,
                }
            })
            .collect();

        Some(Self {
            entries,
            saved: true,
        })
    }

    pub fn add(&mut self, new_entry: String) {
        self.saved = false;
        self.entries
            .retain(|old_entry| old_entry.string != new_entry);
        self.entries.push(Entry {
            string: new_entry,
            last_used: SystemTime::now(),
        });
    }

    pub fn save(&mut self) {
        self.trim_old();

        let path = match data_path() {
            Some(path) => path,
            None => return,
        };

        let file = match fs::File::create(path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut writer = BufWriter::new(file);
        for entry in &self.entries {
            let secs_since_epoch = entry
                .last_used
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            writer
                .write_all(secs_since_epoch.to_string().as_bytes())
                .unwrap();
            writer.write_all(b" ").unwrap();
            writer.write_all(entry.string.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
        writer.flush().unwrap();

        self.saved = true;
    }

    /// Removes entries older than half a year
    fn trim_old(&mut self) {
        let threshold = Duration::from_secs(60 * 60 * 24 * 365 / 2);
        self.entries
            .retain(|entry| entry.last_used.elapsed().unwrap() < threshold);
        self.saved = false;
    }
}

impl Drop for EntrySet {
    fn drop(&mut self) {
        if !self.saved {
            self.save();
        }
    }
}
