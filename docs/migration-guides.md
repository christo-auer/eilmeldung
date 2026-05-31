
# Breaking Changes and Migration Guides

Here you can find migration guides for breaking changes.

**IMPORTANT**: Don't copy `example/default-config.toml`. Start with an **empty** `config.toml` and just add the settings you want to be **different** from the `default-config.toml`! `default-config.toml` is for **reference purposes** only and is prone to break if used directly.

## With 1.5.3
### `config.toml`


- configuration option `all_label` was renamed to `feeds_label`
- all `*_icon` config operations were moved into the subsection `[icon_set]` with the `_icon` suffix stripped. for instance, `unread_icon` is now `unread` in `[icon_set]` (or just `icon_set.unread`);  see [configuration.md#icons-set](here)
- some `_label` settings contained icons inline. there is a new placeholder `{icon}` and a corresponding icon which is defined in `[icon_set]`
- new option `preset` in `[icon_set]` which can either be `nerd` or `ascii` to predefine a set of icons; individual icons can be overwritten
