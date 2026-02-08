# Unreleased

- from 0.9.0: **NEW**: Automatic article operations after synchronization!
  - want to automatically set mark ads or paywall articles to read?
  - or add tags to articles with certain keywords? Or from your favorite feeds?
  - this is now possible with the new option `after_sync_commands`!
  - read more in `docs/configuration.md`, *After-Sync Commands*
- hotfixes
  - sensible selection in article list after a article has been marked as read
  - attempt relogin after logged out

# 0.9.0 - 2026-02-07

- **NEW**: Automatic article operations after synchronization!
  - want to automatically set mark ads or paywall articles to read?
  - or add tags to articles with certain keywords? Or from your favorite feeds?
  - this is now possible with the new option `after_sync_commands`!
  - read more in `docs/configuration.md`, *After-Sync Commands*

# 0.8.4 - 2026-02-04

- **Breaking Changes**
  - `thumbnail_width` is now a *dimension* value, default is `thumbnail_width = "14 length"` (14 columns)
  - `thumbnail_height` is a *dimension* value which defines the height space reserved for the thumbnail, default is `thumbnail_height = "5 length"` (5 rows)
- bugfix: `libxml` error output messed up TUI; workaround which temporarily redirects `stderr` when scraping an article
- improvement: default keybinding of `r` now selects next unread article (instead of jumping to the top)
- status bar eye candy

# 0.8.3 - 2026-01-30

- hotfix: removed hash for `news_flash` from nix package declaration


# 0.8.2 - 2026-01-30

- new CLI magic for the people
  - `--sync` syncs all feeds and outputs (customizable) sync statistics (for you `cron`/`systemd` tweakers)
  - `--import-opml`/`--export-opml` for importing/exports opml directly from the command line for automatic setup; combine with login settings and you can setup `eilmeldung` without manual input!
  - `--logout` to logout
  - for more information checkout `docs/cli_args.md`
  - **Warning**: You should not execute these commands while eilmeldung is running in another process to avoid data inconsistency!
- **inoreader** (OAuth2) provider is now supported (**note**: You need a Pro account to create an application token to grand `eilmeldung` access to **inoreader**)
- bugfixes
  - deleting categories works now (thanks to @JanGernert for updating `news_flash` to 3.0.0)
  - fixed false negatives in reachability checks (thanks to @janbuchar)
  - re-login to prevent auth errors on sync attempts (thanks to @janbuchar)

# 0.8.1 - 2026-01-28

- two new component style modifiers
  - `unread_count` is applied to unread count label in feed tree
  - `marked_count` is applied to marked count label in feed tree
  - default for both is `{ mods = ["italic"]}`; if you want the old style set them to `{ }` (no modification of style)
- feed list now tries to make sure that the "most sensible" item is selected after changes in the tree (in particular after items have been set to read)
- bugfix: feed list now always handles node collapse/expand commands (not just when focused)

# 0.8.0 - 2026-01-23

- new theming possibilities for unread/read items (**breaking changes**!)
  - settings `unread_modifier` and `article_highlighted` have been **removed**!
  - instead use the component styles `unread`, `read`, `highlighted` and `selected` to adjust how the respective items should look like
  - for more details have a look at *Component Style Modifiers* in `docs/configuration.md`
- new commands
  - `expand`: expands the current item in the tree
  - `expandcategories <scope>`: expand categories with articles in scope, i.e., `all`, `unread` or `marked` articles
  - `collapse`: collapses the current item in the tree
  - `collapseall`: collapses all items in the tree
- bugfix: custom colors all mapped to `none`
- switched to system TLS implementation (thanks to @bgiarrizzo to the PR) which makes it possible to use eilmeldung behind SSL injection proxies

# 0.7.9 - 2026-01-22

- removed share keybindings (`S m`, `S t`, etc.). instead `S` opens the command line with the configured share targets (press TAB to cycle through them)
- bugfixes
  - slightly improved default values for thumbnail scaling, more room to bottom
  - custom colors (for styles) are now properly parsed. 

# 0.7.8 - 2026-01-18

- new configuration options: 
  - set `sync_every_minutes` to periodically sync (default: disabled)
  - `startup_commands` to automatically execute commands on startup, e.g., `startup_commands=["sync", "focus articles"]` to automatically sync on startup and focus the articles list
- new modifier for key bindings: `S-...` for shift, e.g., `S-down` for pressing "shift and downward cursor key"
- bugfix: command input doesn't crash on umlaute/unicode anymore

# 0.7.7 - 2026-01-15

- no more switching back and forth between panels by these new convenient default key bindings:
  - use `Ctrl-j` and `Ctrl-k` to move down/up in the feeds list, from **any panel**
  - use `J` and `K` to scroll the article content down/up (scrape before by pressing `x`)
  - use `M-j` and `M-k` to move down/up in the articles list
- of course, you can customize these key bindings by using the new `in` command
  - `in <panel> <command>` runs a command in the given panel (`feeds`, `articles`, `content`)
  - example: `in feeds down` moves the selection down in the feeds list, `in content gotofirst` scrolls to the top in the article list.
  - the new default key mappings are:
    ```toml
    "C-j"       = ["in feeds down"]
    "C-k"       = ["in feeds up"]
    "J"         = ["in content down"]
    "K"         = ["in content up"]
    "M-j"       = ["in articles down"]
    "M-k"       = ["in articles up"]
    ```
  - note: if you want the old default key mapping for `J` back, add `"J" = ["read", "nextunread"]` to `[input_config.mappings]` in `config.toml`
  - switched to a different network connectivity library (`if-watch`)


# 0.7.6 - 2026-01-13

- bugfix: when in zen mode (distraction-free mode) and a modal dialog is active (confirmation, etc.), zen mode is temporarily deactivated

# 0.7.5 - 2026-01-12

- **new feature**: feed list now supports showing only items with unread or marked items (or all)
  - press `1`, `2` or `3` *in the feed list* to show *all* or items with *unread* or *marked* articles in the feed list
  - of course, this still works in the *article list* 
  - the new setting `feed_list_scope` defines the default setting on startup
  - if you want to synchronize what the feed list and article list show, have a look at the FAQ (*Features and Capabilities*)
  - new display of current scope using only icons
- bugfix: feed list now tries to restore selection after changes (instead of selecting nothing)

# 0.7.4 - 2026-01-10

- fixed bug which made `default-configuration.toml` invalid; now the style set works as documented
- new option: `hide_default_sort_order`, if `true`, hides the sort order if the default sort order is applied

# 0.7.3 - 2026-01-07

- bugfix: feeds imported via OPML in the root level could not be yanked (`c y`). This works now.

# 0.7.2 - 2026-01-07

- bugfix: eilmeldung wouldn't launch if no `config.toml` exists; now it launches with the default configuration


# 0.7.1 - 2026-01-06

- bugfix: when entering a key sequence with multiple alternatives, if there is one which already matches, pressing enter (default key binding) executes the key binding immediately. Otherwise there is a timeout running down after which it is executed. If escape is pressed, the keybinding input is aborted.

# 0.7.0 - 2026-01-05

- **new feature**: you can now **sort** the article list
  - via the new `sort` command: `sort <feed >date` sorts the articles by feed ascending and from oldest to newest (see command documentation for details)
  - define a *default sort* order via the configuration option `default_sort_order` (default value is `<date`, i.e., from newest to oldest)
  - use the new `sort="..."` in article queries, e.g., `#readlater unread sort="<feed <date"` queries all unread articles with tag `readlater` and sorts them first by `feed` then from newest to oldest
  - new default key bindings
    - `\` opens the command line with `sort`
    - `| r` clears the current sort and reverts to default sort ordre (or query sort order)
    - `| |` reverses the current sort order
    - as always, you can define your own key bindings to your desires
# 0.6.2 - 2026-01-05

- there is now an explicit error when `config.toml` is invalid (e.g., duplicate entries)
- `C-u` was a duplicate mapping in `default-config.toml`
- some remappings in `default-config.toml`:
  - `M-u` maps to `cmd unread` (before `C-u`)
  - `M-m` maps to `cmd mark`
  - `M-v` maps to `cmd unmark` (before `C-v`)
  - `M-r` maps to `cmd read` (before `C-r`)

# 0.6.1 - 2026-01-03

- added MUSL CI targets

# 0.6.0 - 2026-01-03

- added new config section: `[login_setup]` for automatic login

# 0.5.2 - 2025-12-31

- added example for light theme
- made default theme more consistent between light/dark themes

# 0.5.1 - 2025-12-30
- fixed bug (issue 55): arguments to command are now not quoted anymore

# 0.5.0 - 2025-12-30

- **Breaking Changes**: 
  - The followng layout options have been replaced by a more flexibe options:
  ```
  feed_list_width_percent
  article_list_width_percent
  article_list_height_lines
  ```
  - They have been replaced by
  ```
  feed_list_focused_width
  article_list_focused_width
  article_list_focused_height
  article_content_focused_height
  ```
  - see configuration documentation and section *Layout Configuration*
- content view no shows scraped (full) article content if available. press `x` (command `scrape`) to retrieve full article. I won't implement an automatic scrape to reduce load on websites.


# 0.4.11 - 2025-12-28

- new share target: external command. You can now define a new share target by defining a shell command to which the URL and title of the article is passed
- throbber, which indicates a running process, is now more visible
- bug fixed: window is now redrawn when the terminal window or font is resized

# 0.4.10 - 2025-12-27

- AUR packages `eilmeldung` and `eilmeldung-git` now available

# 0.4.9 - 2025-12-26

- optimized amount of redraw calls for lower CPU consumption 
- tags are now visible again in content view

# 0.4.8 - 2025-12-25

- placeholder image is now shown when thumbnail could not be loaded
- fixed wrong display of title (and other elements) when a html-escaped code used wide ampersand instead of a regular ampersand
- article content now also shows author
- the option `keep_articles_days` (default 30) sets the amount of days before articles are removed

# 0.4.7 - 2025-12-24

- automatically fetching feed after adding, squashed some related bugs
- optimized logic for sensing if service is reachable, leads to faster reconnect on unstable networks or after disconnects
- after service is reachable again (after a disconnect), the reqwest client is rebuilt. this is need for some providers to accept the connection after a reconnect (e.g. inoreader)

# 0.4.6 - 2025-12-23

- scrollbar appearance is now configurable

# 0.4.5 - 2025-12-22

- added spaces between tags in content display
- added sexy scrollbars
- removed scroll amount displays from article list and content
# 0.4.4 - 2025-12-21

- refactored content layout to be more consistent

# 0.4.3 - 2025-12-21

- HTML/markdown content now renders if `content_preferred_type` is set to `markddown`
- `content_preferred_type`'s settings changed to `plain_text` and `markdown`, example updated

# 0.4.2 - 2025-12-20

- zen/distraction-free mode now shows no summary/thumbnail

# 0.4.1 - 2025-12-20

- fixed: scrolling now works on filtered content in help dialog

# 0.4.0 - 2025-12-20
- command `helpinput` (default keybinding `?`) now shows a popup with all key bindings which can also be search (default keybinding `/`)
- new input-related commands: `submit`, `abort`, `clear` applicable for situations where a user input is required (e.g. command line or search)
- new input-related command: `find` depending on context, open a search input (default keybinding `/`)

# 0.3.0 - 2025-12-18

- added cli arguments (see `docs/cli_args.md`)

# 0.2.1 - 2025-12-18

- remove clang dep from homebrew formula

# 0.2.0 - 2025-12-17
- added tag matching without `tag:`, i.e., use `#tag` instead of `tag:#tag`
- switched to homebrew release from source
- fixed wrong syntax in default config (input mappings need arrays)
- removed md marker from default config
- added instructions for homebrew installation
# 0.1.0 - 2025-12-14
- initial release. see `README.md`

