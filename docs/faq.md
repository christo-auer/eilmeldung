# Frequently Asked Questions (FAQ)

Common questions and answers about eilmeldung.

---

## Table of Contents

- [General Questions](#general-questions)
- [Features & Capabilities](#features--capabilities)
- [Configuration & Customization](#configuration--customization)
- [Troubleshooting](#troubleshooting)

---

## General Questions

### Which providers are supported?

See [news_flash_gtk for all supported providers](https://gitlab.com/news-flash/news_flash_gtk).

**Note**: inoreader is currently **NOT** directly supported. Create an issue if you need support for inoreader!

### What does "eilmeldung" mean?

*eilmeldung* is German for *breaking news*.

### Do I need a Nerd Font?

Yes, you need a [Nerd Font](https://github.com/ryanoasis/nerd-fonts) compatible font/terminal for icons to display correctly. Without it, some icons may appear as boxes or question marks.

---

## Features & Capabilities

### Can I call an external program with the URL/title of the current article?

Yes! You can define *custom share targets* which accept commands or URLs. For instance, in the configuration file:

```toml
share_targets = [
  'hackernews https://news.ycombinator.com/submitlink?u={url}&t={title}', # opens webbrowser
  'sendmail ./sendmail.sh me@eilmeldung.org "{title}" "{url}"', # passes title and URL to shell script
  # more share targets
]
```

In `eilmeldung`, select an article and share it with `share hackernews` or `share sendmail` (use TAB for autocompletion). Of course, you can also define key bindings for this:

```toml
[input_config.mappings]
"S h" = ["share hackernews"]
"S m" = ["share sendmail"]
```

See [Share Target Configuration](configuration.md#share-target-configuration) for details and examples.

### Does eilmeldung support smart folders?

Yes, by using queries in the feed list. For example:

```toml
feed_list = [
  'query: "Important Today" #important unread today', 
  # ... all other entries you want to have in the feed list
]
```

Creates an entry *Important Today* in the feed list which lists all unread articles with the tag `#important` from today. 

See [Article Queries](queries.md) and [Feed List Configuration](configuration.md#feed-list-configuration) for more.

### How can I save articles for reading later?

You can define a tag for this:

```
:tagadd readlater red
```

Then define a keybinding to quickly tag an article:

```toml
[input_config.mappings]
"R" = ["tag readlater"]
# or if you want to navigate to the next unread article after tagging
"R" = ["tag readlater", "nextunread"]
```

And finally create a query in the feed list for quick access:

```toml
feed_list = [
  'query: "Read Later" #readlater unread',
  # ... all other entries you want to have in the feed list
]
```

### Can I sync what the feed list and article list how (all/unread/marked)?

There is no dedicated setting for this but this can be achived by using the same value for `article_scope` and `feed_list_scope` and remapping the keybindings for changing the scope:

```toml
[input_config.mappings]
"1" = ["show feeds all", "show articles all"]
"2" = ["show feeds unread", "show articles unread"]
"3" = ["show feeds marked", "show articles marked"]
```

With this the article list and feed list always have the same scope.

### Can I sort the feeds and categories?

Yes, *yank* the element you want to move (`c y`), move to the position you want to insert the element and press `c p` to insert *after* and `c P` to insert before the selected element.

---

## Configuration & Customization

### Can I adjust the layout and panel sizes?

Yes! Have a look at the section *Layout* in [Configuration](configuration.md#layout-configuration).

You can control:
- Feed list width when focused
- Article list width and height when focused
- Article content height when focused

This allows you to create static layouts (all panels same size) or dynamic layouts (focused panel gets more space).

### Is there a light color palette?

The default color palette is dark. For a light palette using ANSI 16 colors, see [this example file](../examples/light-ansi-palette.toml) and insert it into your `config.toml`.

You can also customize all colors individually. See [Theme Configuration](configuration.md#theme-configuration).

### Can I configure login information via the configuration file?

Yes! Check out *Automatic Login* in [Configuration](configuration.md#automatic-login).

This allows you to:
- Skip the interactive login process
- Store credentials securely using password managers
- Automate setup for multiple instances

**Important**: Use the `cmd:` prefix to call a password manager instead of storing passwords in plain text:

```toml
[login_setup]
password = "cmd:pass my-passwords/eilmeldung"
```

### How do I find the right login settings?

Run `eilmeldung --print-login-data` to see the configuration needed for your setup. This will guide you through the login process and output the configuration at the end.

See [Finding the Right Settings](configuration.md#finding-the-right-settings) for details.

### Do I always have to focus a panel to move/execute a command there?

No, the `in <panel> <command>` executes the command in the panel (`feeds`, `articles`, `content`). For instance, to move down in the feed list, the command is `in feeds down`; use this for customizing your key bindings!

---

## Troubleshooting

### Icons don't display correctly

Make sure you're using a [Nerd Font](https://github.com/ryanoasis/nerd-fonts) compatible font in your terminal. See [Installation](installation.md#important-nerd-fonts).

### How do I enable debug logging?

Run eilmeldung with debug logging to troubleshoot issues:

```bash
eilmeldung --log-level DEBUG --log-file ~/eilmeldung-debug.log
```

See [Command Line Arguments](cli_args.md) for more options.

---

## Still Have Questions?

- Check the full [Documentation](../README.md#documentation)
- Review the [Commands](commands.md) reference
- Read the [Configuration](configuration.md) guide
- Report issues at [GitHub](https://github.com/christo-auer/eilmeldung/issues)
