# Mahou
## Magically easy anime downloader

## How to use
Either download [the latest release](https://github.com/LeoRiether/mahou/releases/latest) and
run the executable, or build it from source by running

```bash
cargo install mahou
mahou
```

That's it! If it doesn't find the `mahou` executable, try adding `~/.cargo/bin`
to your `$PATH`.

You should also set an alias for mahou with your preferred settings, for example:

```bash
# ~/.bashrc
alias mahou="mahou --res 1080p --directory $HOME/Downloads"
```

## Automation
You can configure mahou to automatically download anime without user input, for
example to download the latest episode of a show in a cronjob. To do this,
you're going to have to use almost every flag:

```bash
mahou --search "Name of the show" --episode latest \
      --res 1080p --directory $HOME/Downloads/Seasonal \
      --filter "holland ipv6" --download-first
```

## T-thanks
Heavily inspired by [anime-cli](https://github.com/DeGuitard/anime-cli) (if it
was a library I would have used it instead of... copying code from it... :/)
