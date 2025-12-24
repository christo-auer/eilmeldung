# Unreleased

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

