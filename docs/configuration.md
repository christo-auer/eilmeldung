# Configuration 

**eilmeldung** uses a TOML configuration file to customize behavior, appearance, and key bindings. The configuration file is optional, eilmeldung works out-of-the-box with sensible defaults.
The configuration file location is `~/.config/eilmeldung/config.toml`.

**Note:** Icons and special characters require a terminal and font that support [Nerd Fonts](https://www.nerdfonts.com/).

You can find the default configuration in `examples/default-config.toml`


## Basic Configuration Options


| Option                            | Type                | Description                                                |
| --------                          | ------              | -------------                                              |
| `refresh_fps`                     | integer             | UI refresh rate in frames per second                       |
| `network_timeout_seconds`         | integer             | timeout for network operations                             |
| `article_scope`                   | string              | Default article scope: `"all"`, `"unread"`, or `"marked"`  |
| `default_sort_order`              | string (sort order) | Default sort order for articles: e.g., `"date"`, `">date"`, `"feed date"` (see Article Queries for syntax) |
| `keep_articles_days`              | integer             | amount of days before articles are removed                 |
| `offline_icon`                    | char                | Icon displayed when offline                                |
| `read_icon`                       | char                | Icon for read articles                                     |
| `unread_icon`                     | char                | Icon for unread articles                                   |
| `marked_icon`                     | char                | Icon for marked articles                                   |
| `unmarked_icon`                   | char                | Icon for unmarked articles                                 |
| `tag_icon`                        | char                | Icon for tags                                              |
| `command_line_prompt_icon`        | char                | Icon for command line prompt                               |
| `scrollbar_begin_symbol`          | char                | Symbol at top of scrollbars                                |
| `scrollbar_end_symbol`            | char                | Symbol at bottom of scrollbars                             |
| `scrollbar_thumb_symbol`          | char                | Symbol placed at current position of scrollbars            |
| `scrollbar_track_symbol`          | char                | Symbol placed between top and bottom of scrollbars         |
| `all_label`                       | string              | Label format for "All" in feed list                        |
| `feed_label`                      | string              | Label format for feeds                                     |
| `category_label`                  | string              | Label format for categories                                |
| `categories_label`                | string              | Label format for categories section                        |
| `tags_label`                      | string              | Label format for tags section                              |
| `tag_label`                       | string              | Label format for individual tags                           |
| `query_label`                     | string              | Label format for query items                               |
| `article_table`                   | string              | Article list column format                                 |
| `date_format`                     | string              | Date format (strftime syntax)                              |
| `articles_after_selection`        | integer             | Number of articles to show after selection                 |
| `auto_scrape`                     | boolean             | Automatically scrape full article content                  |
| `thumbnail_show`                  | boolean             | Show article thumbnails                                    |
| `thumbnail_width`                 | integer             | Thumbnail width in characters                              |
| `thumbnail_resize`                | boolean             | Resize thumbnails to fit (**this may cause slowdowns**)    |
| `thumbnail_fetch_debounce_millis` | integer             | Delay before fetching thumbnail (ms)                       |
| `text_max_width`                  | integer             | Maximum text width for article content                     |
| `content_preferred_type`          | string              | Preferred content type: `"PlainText"` or `"Markdown"`      |
| `feed_list_focused_width`         | dimension           | Width of feed list when focused                            |
| `article_list_focused_width`      | dimension           | Width of article list when focused                         |
| `article_list_focused_height`     | dimension           | Height of article list when focused                        |
| `article_content_focused_height`  | dimension           | Height of article content when focused                     |


**Label Placeholders:**
- `{label}`: Item name
- `{unread_count}`: Number of unread articles

**Article Table Columns:**
- `{read}`: Read/unread icon
- `{marked}`: Marked/unmarked icon
- `{tag_icons}`: Tag icons
- `{age}`: Article age/date
- `{title}`: Article title

**Dimension:** Is a string:
- **Percentage**: `"n%"` where `n` is a number from 1 to 100, e.g., `"33%"`, meaning 33% of the available width/height
- **Length**: `"n length"` where `n` is a positive, e.g., `"10 length"`, meaning 10 rows (height)  or 10 columns (width)


---

## Default Sort Order

You can configure the default sort order for articles using the `default_sort_order` option. This sort order is applied whenever articles are displayed, unless overridden by a query-specific sort order or an adhoc sort command.

**Syntax:**
```toml
default_sort_order = "<sort order>"
```

**Examples:**
```toml
# Sort by date, newest first (common for RSS readers, this is the default)
default_sort_order = "date"

# Sort by feed name, then by date (newest first within each feed)
default_sort_order = "feed date"

# Sort by title alphabetically (case is ignored)
default_sort_order = "title"

# Sort by date oldest first
default_sort_order = ">date"
```

For complete sort order syntax and available sort keys, see [Commands](commands.md#sorting-articles).

**Default Value:** `"date"` (newest first)

---

## Input Configuration

Input configuration is defined in the `[input_config]` section.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `scroll_amount` | integer | `10` | Number of lines to scroll with page up/down |
| `timeout_millis` | integer | `5000` | Timeout for multi-key sequences (milliseconds) |
| `mappings` | table | See below | Key binding mappings |

### Keybinding Customization

Key bindings are defined in the `[input_config.mappings]` section as key-value pairs. A (sequence of) key(s) is mapped onto an array of commands.

**Key Syntax:**
- Single keys: `"a"`, `"j"`, `"k"`
- Control keys: `"C-c"` (Ctrl+C), `"C-r"` (Ctrl+R)
- Special keys: `"space"`, `"tab"`, `"backtab"`, `"up"`, `"down"`, `"left"`, `"right"`, `"esc"`, `"enter"`
- Multi-key sequences: `"g g"` (press g twice), `"c w"` (c then w)

**Examples:**
```toml
[input_config.mappings]
# Single command
"q" = ["quit"]

# Multiple commands executed in sequence
"o" = ["open", "read", "nextunread"]

# Multi-key sequences
"g g" = ["gotofirst"]
"g t" = ["focus feeds"]

# Unbind a key
"x" = ["nop"]
```

For a complete list of available commands, see the Commands section. For default keybindings, see the main page or execute the command `helpinput`.

---

## Theme Configuration

Theme configuration is defined in the `[theme]` section with two subsections: `color_palette` and `style_set`.

### Color Palette

The color palette defines the base colors used throughout the application. Colors can be specified as color names or hex codes (see [ratatui Color documentation](https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html) for all options).

| Option | Default (ANSI) | Description |
|--------|----------------|-------------|
| `background` | `"black"` | Background color |
| `foreground` | `"white"` | Foreground/text color |
| `muted` | `"dark_gray"` | Muted/inactive elements |
| `highlight` | `"yellow"` | Highlighted elements |
| `accent_primary` | `"magenta"` | Primary accent (feeds, borders) |
| `accent_secondary` | `"blue"` | Secondary accent (categories) |
| `accent_tertiary` | `"cyan"` | Tertiary accent (tags) |
| `accent_quaternary` | `"yellow"` | Quaternary accent (queries) |
| `info` | `"magenta"` | Info messages |
| `warning` | `"yellow"` | Warning messages |
| `error` | `"red"` | Error messages |

**Example:**
```toml
[theme.color_palette]
background = "#1e1e2e"
foreground = "#cdd6f4"
accent_primary = "#f5c2e7"
accent_secondary = "#89b4fa"
```

### Component Styles

Component styles define how UI elements appear. Each component can have:
- `fg`: Foreground color (from palette or custom)
- `bg`: Background color (from palette or custom)
- `mods`: Array of modifiers

**Color References:**
- Palette colors: `"background"`, `"foreground"`, `"muted"`, `"highlight"`, `"accent_primary"`, `"accent_secondary"`, `"accent_tertiary"`, `"accent_quaternary"`, `"info"`, `"warning"`, `"error"`
- Custom colors: `{ custom = "#ff0000" }` or `{ custom = "red" }`

**Available Modifiers:**
`"bold"`, `"dim"`, `"italic"`, `"underlined"`, `"slow_blink"`, `"rapid_blink"`, `"reversed"`, `"hidden"`, `"crossed_out"`

| Component | Default FG | Default BG | Default Mods | Description |
|-----------|-----------|------------|--------------|-------------|
| `header` | `accent_primary` | `none` |: | Section headers |
| `paragraph` | `foreground` | `none` |: | Regular text |
| `article` | `foreground` | `none` |: | Article items |
| `article_highlighted` | `highlight` | `none` | `["bold"]` | Selected article |
| `feed` | `accent_primary` | `none` |: | Feed items |
| `category` | `accent_secondary` | `none` |: | Category items |
| `tag` | `accent_tertiary` | `none` |: | Tag items |
| `query` | `accent_quaternary` | `none` |: | Query items |
| `yanked` | `highlight` | `none` | `["reversed"]` | Yanked items (for moving) |
| `border` | `muted` | `none` |: | Panel borders |
| `border_focused` | `accent_primary` | `none` |: | Focused panel border |
| `statusbar` | `background` | `accent_primary` |: | Status bar |
| `command_input` | `foreground` | `muted` |: | Command line input |
| `inactive` | `muted` | `none` |: | Inactive elements |
| `tooltip_info` | `background` | `info` |: | Info tooltips |
| `tooltip_warning` | `background` | `warning` |: | Warning tooltips |
| `tooltip_error` | `background` | `error` |: | Error tooltips |
| `unread_modifier` |: |: | `"bold"` | Modifier applied to unread items |

**Example:**
```toml
[theme.style_set]
article_highlighted = { fg = "highlight", bg = { custom = "#2a2a3a" }, mods = ["bold", "italic"] }
border_focused = { fg = "accent_primary", mods = ["bold"] }
feed = { fg = { custom = "#f5c2e7" } }
unread_modifier = "bold"
```

---

## Feed List Configuration

The `feed_list` array defines what appears in the feed list panel and in what order. Each entry is a string that specifies the type and display format.

**Syntax:**
- `"feeds"`: Show feeds as a tree (hierarchical)
- `"categories"`: Show categories as a tree
- `"tags"`: Show tags as a tree
- `"* feeds"`: Show feeds as a flat list (prefix with `*`)
- `"* categories"`: Show categories as a flat list
- `"* tags"`: Show tags as a flat list
- `'query: "<label>" <query>'`: Custom query with label (label must be in double quotes)

**Default:**
```toml
feed_list = [
  'query: "Today Unread" today unread',
  'query: "Today Marked" today marked',
  "feeds",
  "* categories",
  "tags",
]
```

**Custom Example:**
```toml
feed_list = [
  'query: "Urgent" marked #urgent',
  'query: "This Week" newer:"1 week"',
  'query: "Tech News" feed:/tech/ unread',
  "feeds",                    # Hierarchical feed tree
  "* tags",                   # Flat tag list
]
```

---

## Share Target Configuration

The `share_targets` array defines available sharing targets. Each entry can be a built-in target name or a custom target definition.

**Built-in Targets:**
- `"clipboard"`: Copy URL to clipboard
- `"reddit"`: Share on Reddit
- `"mastodon"`: Share on Mastodon
- `"telegram"`: Share on Telegram
- `"instapaper"`: Save to Instapaper

**Custom Target Syntax:**
`'<name> <template>'` where:
- `<name>`: Target name used in commands (not quoted, a single word)
- `<template>`: any occurrence of `{url}` is replaced by the URL of the article and any `{title}` is replaced by its title
  - **Sharing via Webbrowser**: if the template starts with `http://...` or `https://...` the template is interpreted as a web URL and upon sharing the webbrowser is opened with the given URL
  - **Sharing to a Shell Command**: otherwise the template is interpreted as a shell command with arguments. **Note**: 
    - A new process is spawned in the background whose with `stdin`, `stdout`, and `stderr` redirected to `null`. In particular, don't that you see any output.
    - This does not support any shell features like input output redirection (`>`, etc.), pipes (`|`) or other advanced shell features. Also no shell variables are replaced (`~`, `$HOME`). If you want more sophisticated behaviour, create a shell script and call the shell script.

**Default:**
```toml
share_targets = [
  "clipboard",
  "reddit",
  "mastodon",
  "instapaper",
  "telegram",
]
```

**Custom Example:**
```toml
share_targets = [
  "clipboard",
  "reddit",
  'hackernews https://news.ycombinator.com/submitlink?u={url}&t={title}',
  'pocket https://getpocket.com/save?url={url}&title={title}',
  'sendmail ./sendmail.sh me@eilmeldung.org \"{title}\" \"{url}\"', # note the double quotes around the two variables
  'chromium chromium \"{url}\"',
]
```

## Layout Configuration

You can adjust the layout, that is, the size of the different panels when they are focused and unfocused by the following variables:

- `feed_list_focused_width`: width of feed list when focused
- `article_list_focused_width`: width of article list when focused
- `article_list_focused_height`: height of article list when focused
- `article_content_focused_height`: height of article content when focused

Each has a *dimension* value whih is a string, e.g., `"10 length"` for ten rows/columns or `"33%"` for 33% of the available width/height. For instance, if the feed list should occupy 25% of the total width when focused, set its value to `"25%"` and if you want have 10 articles visible in the article list, set its height value to `"11 length"` (+1 for the header).


### Example: Static Layout (default)

With the default values, the width/height of each panel is fixed. For example, the feed list is always 25% of the whole width regardless of whether it is focused or not.

```toml
feed_list_focused_width = "25%"
article_list_focused_width = "75%"
article_list_focused_height = "20%"
article_content_focused_height = "80%"
```

https://github.com/user-attachments/assets/c4e6e89d-e95e-4a80-b660-5e1b982f6108

### Example: Dynamic Layout

Here is an example of values, where unfocused panels are smaller to give more space to the focused panel:

```toml
feed_list_focused_width = "33%"
article_list_focused_width = "85%"
article_list_focused_height = "66%"
article_content_focused_height = "80%"s
````

https://github.com/user-attachments/assets/ffc51e67-1842-4b49-a798-6a5d65b04265

### Example: Fully Dynamic Layout

Here is an example where there feed list completely vanishes when the article list is focused, and the article list completely vanishes when the content is focused:

```toml
feed_list_focused_width = "33%"
article_list_focused_width = "100%"
article_list_focused_height = "66%"
article_content_focused_height = "100%"
```

https://github.com/user-attachments/assets/e9277d94-a6da-49de-8dd0-8c6a75e09430

## Automatic Login

Upon first starting `eilmeldung`, the user is asked to enter login information after which `eilmeldung` logs into the provider and syncs the content. This interactive login setup can be *automated* by filling the section `[login_setup]`. The settings are:


| Option                | Type   | `login_type`                               | Description                                                                   |
| ---                   | ---    | ---                                        | ---                                                                           |
| `login_type`          | string |                                            | Type of login: `"no_login"`, `"direct_password"`, `"direct_token"`, `"oauth"` |
| `provider`            | string | all                                        | Provider: `"local_rss"`, `"freshrss"`, etc.                                   |
| `url`                 | string | `oauth` (required); `direct_password`, `direct_token` (optional) | URL for connection |
| `user`                | string | `direct_password`                          | Username for direct login                                                     |
| `password`            | secret | `direct_password`                          | Password or command which produces password  (see below!)                     |
| `token`               | secret | `direct_token`                             | Token for login by token                                                      |
| `oauth_client_id`     | string | `oauth`                                    | *Optional*: client ID for oauth login (see note below)                        |
| `oauth_client_secret` | secret | `oauth`                                    | *Optional*: client secret for oauth login (see note below)                    |
| `basic_auth_user`     | string | `direct_password`, `direct_token`          | *Optional*: user name for http basic authentication |
| `basic_auth_password` | secret | `direct_password`, `direct_token`          | *Optional*: password for http basic authentication |

**Note:** For OAuth login, if you provide custom OAuth credentials, both `oauth_client_id` and `oauth_client_secret` must be provided together. You cannot provide only one of them. If you want to use the provider's default OAuth credentials, omit both fields.

**Overwhelmed?** Check *Finding the Right Settings* below! But first read about:

### Secrets

Configuration options with type *secret* are strings which

- either contain the secret itself (e.g, `password = "abcd1234" `); storing password in *clear text* is **NOT RECOMMENDED**
- or contain a command with prefix `cmd:` which outputs the secret on its output (e.g., `password = "cmd:pass my-passwords/eilmeldung"`); **THIS IS THE WAY**

### Finding the Right Settings

`eilmeldung` outputs all needed values via the command line switch `--print-login-data`. If you are already logged in, it simply outputs the login data. If you are not logged in, you will be led through the interactive login process and the login data is output afterwards:

```bash
eilmeldung --print-login-data

Welcome to +++ eilmeldung +++
...
...
> Are you satisfied with these settings? Select `n` to change them. Yes
Attempting to login and synchronize...
login and initial sync successful

login_type = "direct_password"
provider = "freshrss"
user = "chris"
url = "http://x.y.z.w/api/greader.php/"
password = "*******"
```

Note that the password is *redacted*! You have to replace the contents of `password`  with your actual password command. If you, for some reason, want to output the password values verbatim, add the command line switch `--show-secrets`.
Simply copy and paste this into you `config.toml`.

```toml
[login_setup]
login_type = "direct_password"
provider = "freshrss"
user = "username"
url = "http://x.y.z.w/api/greader.php/"
password = "cmd:pass my-passwords/eilmeldung"
```

