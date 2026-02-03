# Key Bindings Reference

This document provides a comprehensive reference of all default key bindings in eilmeldung.

**Note:** You can redefine all key bindings according to your preferences. You can also add completely new key bindings via the [configuration file](configuration.md).

---

## Table of Contents

- [Getting Help](#getting-help)
- [Syncing & Refreshing](#syncing--refreshing)
- [Navigation](#navigation)
- [Reading Articles](#reading-articles)
- [Read/Unread Status](#readunread-status)
- [Marking Articles](#marking-articles)
- [Zen Mode](#zen-mode)
- [Tags](#tags)
- [Article Views](#article-views)
- [Searching & Filtering](#searching--filtering)
- [Sorting Articles](#sorting-articles)
- [Command Line](#command-line)
- [Customizing Key Bindings](#customizing-key-bindings)

---

## Getting Help

| Key | Action |
|-----|--------|
| `?` | Show all available key bindings |

**Tip:** Press `/` in the help dialog to filter and search for specific commands!

---

## Syncing & Refreshing

| Key | Action |
|-----|--------|
| `s` | Sync all feeds with your RSS provider |

---

## Navigation

| Key                   | Action                                                                              |
| -----                 | --------                                                                            |
| `j` / `k`             | Move down / up (vim-style)                                                          |
| `h` / `l`             | Move left (article → article list → feeds) / right (feeds → article list → article) |
| `gg`                  | Go to first item                                                                    |
| `G`                   | Go to last item                                                                     |
| `space`               | Toggle category tree node (open/close)                                              |
| `C-h`/`C-l`           | Navigate up/down in feed tree                                                       |
| `Ctrl-f` / `Ctrl-b`   | Page down / page up                                                                 |
| `Tab` / `Shift-Tab`   | Cycle through panels (forward / backward)                                           |
| `g f` / `g a` / `g c` | Jump directly to feeds / articles / content panel                                   |
| `C-j`                 | Move down in feeds list (from anywhere)                                             |
| `C-k`                 | Move up in feeds list (from anywhere)                                               |
| `J`                   | Move up in article content (from anywhere)                                          | 
| `K`                   | Move up in article content (from anywhere)                                          | 
| `M-j`                 | Move up in article list (from anywhere)                                             |
| `M-k`                 | Move up in article list (from anywhere)                                             | 

---

## Reading Articles

| Key | Action |
|-----|--------|
| `o` | Open article in browser, mark as read, and jump to next unread |
| `O` | Open all unread articles in browser and mark all as read |
| `x` | Scrape full article content from the web (for truncated articles) |
| `S` | Share article (opens command line with share targets) |

---

## Read/Unread Status

| Key | Action |
|-----|--------|
| `r` | Mark current article as read (and go to next unread article) |
| `R` | Mark **all** articles as read (asks for confirmation) |
| `u` | Mark current article as unread |
| `U` | Mark **all** articles as unread (asks for confirmation) |
| `Alt-r` | Open command line to set read with query (e.g., `:read unread today`) |

**Note**: These commands are context-dependent! In the article list, they act on the *current article* or *all articles* in the list. On the feed list/tree they act on the *current category/feed* or *all categories/feeds*.

---

## Marking Articles

| Key | Action |
|-----|--------|
| `m` | Mark current article |
| `M` | Mark **all** articles (asks for confirmation) |
| `v` | Unmark current article |
| `V` | Unmark **all** articles (asks for confirmation) |

**Note**: These commands are context-dependent! In the article list, they act on the *current article* or *all articles* in the list. On the feed list/tree they act on the *current category/feed* or *all categories/feeds*.

---

## Zen Mode

| Key | Action |
|-----|--------|
| `z` | Toggle distraction-free mode (hides all panels except article content) |

---

## Tags

| Key | Action |
|-----|--------|
| `t` | Open command line to tag article (e.g., `:tag tech`), **TAB** to autocomplete tag names |

You can create new tags with `:tagadd urgent red` (press **TAB** for autocomplete colors!). Once created, you can bulk-tag articles: `:tag tech unread` tags all unread articles as `tech`.

---

## Article Views

| Key | Action |
|-----|--------|
| `1` | Show all articles |
| `2` | Show only unread articles |
| `3` | Show only marked articles |

---

## Searching & Filtering

| Key | Action |
|-----|--------|
| `/` | Search articles (type query and press Enter) |
| `n` / `N` | Jump to next / previous match |
| `=` | Open command line to filter articles |
| `+ +` | Apply current filter |
| `+ r` | Clear filter and show all articles |

See [Article Queries](queries.md) for how to craft powerful queries.

---

## Sorting Articles

| Key | Action |
|-----|--------|
| `\` | Open command line to set sort order |
| `\| \|` | Reverse current sort order |
| `\| r` | Clear sort order and restore default |

Articles can be sorted by multiple criteria: `feed`, `date`, `synced`, `title`, or `author`. The sort order is displayed in the article list header with an icon (󰌼 for normal, 󰒿 for reversed).

**Examples:**
```
:sort date                    # Sort by date (newest first)
:sort >date                   # Sort by date (oldest first)
:sort feed title              # Sort by feed name, then by title
:sort feed date               # Sort by feed (A-Z), then by date (newest first)
```

For complete sorting documentation, see [Commands](commands.md#sorting-articles).

---

## Command Line

| Key | Action |
|-----|--------|
| `:` | Open command line for advanced commands |
| `Esc` or `Ctrl-g` | Cancel command input |
| `Ctrl-u` | Clear command input |
| `Tab`, `Backtab` | Trigger/cycle autocomplete and show help |

---

## Customizing Key Bindings

All key bindings can be customized in your configuration file. See the [Input Configuration](configuration.md#input-configuration) section for details on:

- How to remap existing keys
- How to create multi-key sequences (like `gg`)
- How to bind multiple commands to a single key
- How to unbind keys

**Example custom bindings:**
```toml
[input_config.mappings]
# Vim-style save and quit
"Z Z" = ["confirm quit"]

# Quick tagging
"t i" = ["tag important"]
"t u" = ["tag urgent"]

# Custom navigation
"C-j" = ["pagedown"]
"C-k" = ["pageup"]

# Using modifiers
"C-k" = ["..."] # Ctrl
"M-k" = ["..."] # Meta (Alt)
"S-f1" = ["..."] # Shift and F1
"S-down" = ["..."] # Shift and down arrow key
```

See the [default configuration](../examples/default-config.toml) for the complete list of default key bindings.
