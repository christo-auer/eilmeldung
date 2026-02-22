# Command Line Arguments

Run `eilmeldung --help` to display all available command line arguments

---

## Available Arguments

### General

| Argument                          | Description                                                    |
| ---                               | ---                                                            |
| `--log-file <LOG_FILE>`           | Log file (must be writable)                                    |
| `--log-level <LOG_LEVEL>`         | Log level (OFF, ERROR, WARN, INFO, DEBUG, TRACE), default INFO |
| `-c`, `--config-dir <CONFIG_DIR>` | Directory with config files (config.toml, etc.)                |
| `-s`, `--state-dir <STATE_DIR>`   | Directory with state files (database, etc.)                    |
| `-h`, `--help`                    | Print help                                                     |
| `-V`, `--version`                 | Print version                                                  |

### Login Related

| Argument                          | Description                                                    |
| ---                               | ---                                                            |
| `--print-login-data`              | Outputs the login data (for use in `config.toml`)              |
| `--show-secrets`                  | Show secrets (passwords, etc.) in plain text                   |

### Maintenance Actions

All these will exit after finishing.

**Warning**: You should not execute these commands while eilmeldung is running in another process to avoid data inconsistency!


| Argument                    | Description                                                                                         |
| ---                         | ---                                                                                                 |
| `--sync`                    | Sync all feeds and print out sync stats                                                             |
| `--import-opml <OPML-file>` | Import OPML file                                                                                    |
| `--export-opml <OPML-file>` | Export OPML file                                                                                    |
| `--logout`                  | Logout from current provider (**NOTE**: this will **remove** all local data)                        |
| `--quiet`                   | Suppress any output with the actions above                                                          |

### Sync Output

`--sync` outputs sync statistics and its format is defined in `config.toml` in the section `[cli]`.


| Option               | Description                    | Default                   |
| ---                  | ---                            | ---                       |
| `sync_output_format` | Format of a line in the output | `{label}:{count}`         |
| `all_label_format`   | Label of the "all" entry       | `all:All`                 |
| `feed_label_format`  | Label of a feed entry          | `feed:{category}/{label}` |


- In `sync_output_format`:
    - `{label}` is replaced with either the content from `all_label_format` or `feed_label_format`
    - `{count}` is replaced with the amount of newly synced articles
- In `feed_label_format`
  - `{category}` is replaced by the name of the parent category (or the empty string if there is no parent category)
  - `{label}` is replaced by the name of the feed

With the default settings, the output of `--sync` is very `cut`/`awk` friendly:
```
all:All:71
feed:Games/Polygon.com:6
feed:IT-News/Golem.de:6
feed:IT-News/heise online News:7
feed:IT-News/Phoronix:2
feed:Music/Pitchfork:3
feed:Music/The Quietus:2
feed:News/SPIEGEL:15
feed:News/zeit.de:30
```


---

---

## Related Documentation

- [Getting Started](getting-started.md) - Setup and first steps guide
- [Configuration](configuration.md) - Complete configuration file reference
- Main [README](../README.md) - Project overview
