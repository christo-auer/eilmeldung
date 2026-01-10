# Unreleased

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

