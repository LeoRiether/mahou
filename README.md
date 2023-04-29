# Mahou
## Magically easy anime downloader

## How to use
~~Either download [the latest release](https://github.com/LeoRiether/mahou/releases/latest) and
run the executable, or build it from source by running~~
Releases are broken for now, so you'll have to build it yourself:

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


## T-thanks
Heavily inspired by [anime-cli](https://github.com/DeGuitard/anime-cli) (if it was a library I would have used it instead of... copying code from it... :/)
