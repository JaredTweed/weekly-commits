# COSMIC Weekly Commits

A COSMIC desktop applet that mirrors the GNOME Weekly Commits extension from
`weekly-commits-main.zip`.

## Build

```sh
cargo build
```

## Run

```sh
cargo run --bin cosmic-weekly-commits
```

Open settings from the applet popup, or run:

```sh
cargo run --bin cosmic-weekly-commits-settings
```

Settings are stored at `$XDG_CONFIG_HOME/cosmic-weekly-commits/config.json`.
The fallback cache is stored at `$XDG_CACHE_HOME/cosmic-weekly-commits/commits-cache-v1.json`.
