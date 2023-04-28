use argh::FromArgs;
use mahou::finder::{self, EpisodeNumber};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Mahou -- magically easy anime downloader
///
/// Try not using any options for an interactive prompt :)
#[derive(Debug, FromArgs)]
struct Args {
    /// the query to search for
    #[argh(option, short = 'q')]
    search: Option<String>,

    /// the episode to download
    #[argh(option, short = 'e')]
    episode: Option<EpisodeNumber>,

    /// download directory
    #[argh(option, short = 'd')]
    directory: Option<String>,

    /// preferred resolution
    #[argh(option, short = 'r', default = "String::from(\"1080p\")")]
    res: String,
}

fn prompt_search() -> Result<String> {
    Ok(inquire::Text::new("What show would you like to watch today?").prompt()?)
}

fn prompt_episode() -> Result<EpisodeNumber> {
    Ok(inquire::CustomType::<EpisodeNumber>::new("Which episode?")
        .with_default(EpisodeNumber::Latest)
        .with_help_message("Enter a number, 'latest', or 'all' to show all available episodes")
        .prompt()?)
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

    // - Current user input, filter value
    // - Current option being evaluated, with type preserved
    // - String value of the current option
    // - Index of the current option in the original list
    let filter = &|input: &str, _: &finder::Entry, entry: &str, _: usize| {
        let entry = entry.to_lowercase();
        let input = input.to_lowercase();
        input
            .split_whitespace()
            .all(|word| entry.contains(word))
    };

    let selected = inquire::Select::new("Pick an episode", results)
        .with_filter(filter)
        .prompt()?;

    println!("{selected:?}");

    Ok(())
}
