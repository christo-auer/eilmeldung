# Configuration 

**eilmeldung** uses a TOML configuration file to customize behavior, appearance, and key bindings. The configuration file is optional, eilmeldung works out-of-the-box with sensible defaults.
The configuration file location is `~/.config/eilmeldung/config.toml`.

**Note:** Icons and special characters require a terminal and font that support [Nerd Fonts](https://www.nerdfonts.com/).


## Basic Configuration Options

| Option                            | Type    | Description                                               |
| --------                          | ------  | -------------                                             |
| `refresh_fps`                     | integer | UI refresh rate in frames per second                      |
| `article_scope`                   | string  | Default article scope: `"all"`, `"unread"`, or `"marked"` |
| `offline_icon`                    | char    | Icon displayed when offline                               |
| `read_icon`                       | char    | Icon for read articles                                    |
| `unread_icon`                     | char    | Icon for unread articles                                  |
| `marked_icon`                     | char    | Icon for marked articles                                  |
| `unmarked_icon`                   | char    | Icon for unmarked articles                                |
| `tag_icon`                        | char    | Icon for tags                                             |
| `command_line_prompt_icon`        | char    | Icon for command line prompt                              |
| `all_label`                       | string  | Label format for "All" in feed list                       |
| `feed_label`                      | string  | Label format for feeds                                    |
| `category_label`                  | string  | Label format for categories                               |
| `categories_label`                | string  | Label format for categories section                       |
| `tags_label`                      | string  | Label format for tags section                             |
| `tag_label`                       | string  | Label format for individual tags                          |
| `query_label`                     | string  | Label format for query items                              |
| `article_table`                   | string  | Article list column format                                |
| `date_format`                     | string  | Date format (strftime syntax)                             |
| `articles_after_selection`        | integer | Number of articles to show after selection                |
| `auto_scrape`                     | boolean | Automatically scrape full article content                 |
| `thumbnail_show`                  | boolean | Show article thumbnails                                   |
| `thumbnail_width`                 | integer | Thumbnail width in characters                             |
| `thumbnail_resize`                | boolean | Resize thumbnails to fit (**this may cause slowdowns**)   |
| `thumbnail_fetch_debounce_millis` | integer | Delay before fetching thumbnail (ms)                      |
| `text_max_width`                  | integer | Maximum text width for article content                    |
| `content_preferred_type`          | string  | Preferred content type: `"PlainText"` or `"Markdown"`     |
| `feed_list_width_percent`         | integer | Feed list panel width (percentage)                        |
| `article_list_width_percent`      | integer | Article list panel width (percentage)                     |
| `article_list_height_lines`       | integer | Article list height in lines                              |

**Label Placeholders:**
- `{label}`: Item name
- `{unread_count}`: Number of unread articles

**Article Table Columns:**
- `{read}`: Read/unread icon
- `{marked}`: Marked/unmarked icon
- `{tag_icons}`: Tag icons
- `{age}`: Article age/date
- `{title}`: Article title

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
`'<name> <url_template>'` where:
- `<name>`: Target name used in commands
- `<url_template>`: URL with `{url}` and `{title}` placeholders (both are URL-encoded automatically)

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
  'email mailto:?subject={title}&body={url}',
]
```


## Default Configuration

You can find the default configuration in `examples/default-config.toml`

