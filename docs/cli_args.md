# Command Line Arguments

Run `eilmeldung --help` to display all available command line arguments

---

## Available Arguments

| Argument                          | Description                                                    |
| ---                               | ---                                                            |
| `--log-file <LOG_FILE>`           | Log file (must be writable)                                    |
| `--log-level <LOG_LEVEL>`         | Log level (OFF, ERROR, WARN, INFO, DEBUG, TRACE), default INFO |
| `-c`, `--config-dir <CONFIG_DIR>` | Directory with config files (config.toml, etc.)                |
| `-s`, `--state-dir <STATE_DIR>`   | Directory with state files (database, etc.)                    |
| `-h`, `--help`                    | Print help                                                     |
| `-V`, `--version`                 | Print version                                                  |

---

## Common Use Cases

### Running with Debug Logging

Useful when troubleshooting issues or reporting bugs:

```bash
eilmeldung --log-level DEBUG --log-file ~/eilmeldung-debug.log
```

This creates a detailed log file in your home directory that you can share when reporting issues.

### Multiple Instances with Different Configurations

Run separate instances for different RSS providers or configurations:

```bash
# Personal instance with local feeds
eilmeldung --config-dir ~/.config/eilmeldung-personal --state-dir ~/.local/share/eilmeldung-personal

# Work instance with different configuration
eilmeldung --config-dir ~/.config/eilmeldung-work --state-dir ~/.local/share/eilmeldung-work
```

**Tip:** Create shell aliases for quick access:
```bash
alias eil-personal='eilmeldung --config-dir ~/.config/eilmeldung-personal --state-dir ~/.local/share/eilmeldung-personal'
alias eil-work='eilmeldung --config-dir ~/.config/eilmeldung-work --state-dir ~/.local/share/eilmeldung-work'
```

### Testing a New Configuration

Test configuration changes without affecting your main setup:

```bash
# Copy your existing config to a test directory
mkdir -p ~/.config/eilmeldung-test
cp ~/.config/eilmeldung/config.toml ~/.config/eilmeldung-test/

# Run with test configuration
eilmeldung --config-dir ~/.config/eilmeldung-test --state-dir /tmp/eilmeldung-test
```

### Portable Installation

Run eilmeldung from a USB drive or shared directory:

```bash
eilmeldung --config-dir /path/to/portable/config --state-dir /path/to/portable/data
```

---

---

## Related Documentation

- [Getting Started](getting-started.md) - Setup and first steps guide
- [Configuration](configuration.md) - Complete configuration file reference
- Main [README](../README.md) - Project overview
