use argh::FromArgs;
use mahou::{
    autocompleter::{Autocompleter, EntrySet},
    downloader,
    finder::{self, EpisodeNumber},
};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Mahou -- magically easy anime downloader.
/// If --search or --episode are missing, mahou will interactively prompt for them.
#[derive(Debug, FromArgs)]
struct Args {
    /// the show you want to search for
    #[argh(option, short = 's')]
    search: Option<String>,

    /// the episode to download
    #[argh(option, short = 'e')]
    episode: Option<EpisodeNumber>,

    /// directory for the downloaded file. Defaults to current directory
    #[argh(option, short = 'd', default = "\"./\".to_string()")]
    directory: String,

    /// preferred resolution.
    #[argh(option, short = 'r')]
    res: Option<String>,

    /// filter for the results
    #[argh(option, short = 'f')]
    filter: Option<String>,

    /// download the first result instead of prompting to pick one
    #[argh(switch)]
    download_first: bool,
}

fn prompt_search() -> Result<String> {
    let show = inquire::Text::new("What show would you like to watch today?")
        .with_autocomplete(Autocompleter::from_saved_entries())
        .prompt()?;

    if let Some(mut entryset) = EntrySet::from_disk() {
        entryset.add(show.clone());
    }

    Ok(show)
}

fn prompt_episode() -> Result<EpisodeNumber> {
    Ok(inquire::CustomType::<EpisodeNumber>::new("Which episode?")
        .with_default(EpisodeNumber::Latest)
        .with_help_message("Enter a number, 'latest', or 'all' to show all available episodes")
        .prompt()?)
}

fn filter(words: &str, entry: &str) -> bool {
    let entry = entry.to_lowercase();
    let words = words.to_lowercase();
    words.split_whitespace().all(|word| entry.contains(word))
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let search = match args.search {
        Some(search) => search,
        None => prompt_search()?,
    };

    let episode = match args.episode {
        Some(episode) => episode,
        None => prompt_episode()?,
    };

    let results = finder::Query::new(search, args.res, episode).find(&finder::Nibl::default())?;
    let finder::FindResult {
        irc_config,
        mut entries,
    } = results;

    if let Some(f) = &args.filter {
        entries.retain(|entry| filter(f, &format!("{}", entry)));
    }

    if entries.is_empty() {
        eprintln!("No results found :(");
        return Ok(());
    }

    let selected = if args.download_first {
        // Pick first entry
        entries.swap_remove(0)
    } else {
        // Prompt the user to pick an episode

        // - Current user input, filter value
        // - Current option being evaluated, with type preserved
        // - String value of the current option
        // - Index of the current option in the original list
        let inquire_filter = &|input: &str, _: &finder::Entry, entry: &str, _: usize| {
            filter(input, entry)
        };

        inquire::Select::new("Pick an episode", entries)
            .with_filter(inquire_filter)
            .prompt()?
    };

    downloader::download(&selected, irc_config, args.directory)?;

    Ok(())
}
