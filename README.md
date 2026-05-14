# COSMIC Weekly Commits

A COSMIC desktop applet that mirrors the GNOME Weekly Commits extension from
`weekly-commits-main.zip`.

## Install

```sh
just build && just install
```

After installing, log out of your COSMIC session and log back in so the applet is picked up by the desktop shell.

## Update
To apply changes, just pasted this into this directory of the terminal, to update the changes and restart the panel. 
```sh
just build && just install && pkill cosmic-panel
```

## Run Manually

```sh
cargo run --bin cosmic-weekly-commits
```

Open settings from the applet popup, or run:

```sh
cargo run --bin cosmic-weekly-commits-settings
```

Settings are stored at `$XDG_CONFIG_HOME/cosmic-weekly-commits/config.json`.
The fallback cache is stored at `$XDG_CACHE_HOME/cosmic-weekly-commits/commits-cache-v1.json`.
