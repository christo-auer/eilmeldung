# Getting Started with eilmeldung

This guide will help you set up eilmeldung and learn the basics of using it effectively.

---

## Table of Contents

- [First Launch](#first-launch)
- [Initial Setup](#initial-setup)
- [First Steps](#first-steps)
  - [Adding Feeds](#adding-feeds)
  - [Creating Categories](#creating-categories)
  - [Organizing Feeds](#organizing-feeds)
  - [Importing an OPML File](#importing-an-opml-file)
- [Learning the Interface](#learning-the-interface)
- [Next Steps](#next-steps)

---

## First Launch

Once installed, run `eilmeldung` to begin. On first launch, you'll be guided through the setup process.

### Getting Help

Press `?` at any time to see all available key bindings. You can also press `/` in the help dialog to filter for specific commands.

### Command Line

Before we begin: `eilmeldung` has a powerful command line which you can open with `:`. All actions you can trigger in `eilmeldung` are commands and key bindings are just executing one or more commands in the background. Some commands also just open the command line with a predefined command.

**Tip**: Press `Tab` in the command line to trigger autocomplete and see helpful suggestions!

---

## Initial Setup

1. **Choose Your Provider**: Select from local or cloud-based RSS providers ([see news_flash_gtk for all supported providers](https://gitlab.com/news-flash/news_flash_gtk))
2. **Configure Authentication**: Enter your credentials (username/password or token, depending on the provider)
3. **Initial Sync**: The app will sync your feeds from the provider

If you're new to RSS, select **Local** as your provider - it stores everything on your machine without requiring an external service.

---

## First Steps

After setup, you'll want to add some feeds and organize them:

### Adding Feeds

- Press `c f` (command: `feedadd`) to add a new feed - you'll be prompted for the RSS/Atom feed URL
- Example: `:feedadd https://example.com/feed.xml`
- Or with a custom name: `:feedadd https://news.site/rss News Site` (note: no quotes around the name)
- Finding RSS feeds: Many websites provide RSS feeds. Look for an RSS icon or search for "RSS" on the site. You can also use [RSS Lookup](https://www.rsslookup.com/) to find RSS feeds for any website.

### Creating Categories

- Press `c a` to add a new category for organizing your feeds
- Example: `:categoryadd Technology`

### Organizing Feeds

- **Rename**: Select a feed/category and press `c w`, then type the new name
- **Move**: Press `c y` to yank (copy) a feed, navigate to destination, press `c p` to paste after (or `c P` to paste before)
- **Remove**: Press `c d` to remove an empty feed/category, or `c x` to remove with all children

### Importing an OPML File

Instead of manually adding feeds and categories, you can also import an OPML file. An OPML file contains all the categories and feeds you have defined in another RSS reader or provider:

- Export an OPML file from your current provider and save the OPML file somewhere in your home directory.
- In `eilmeldung`, open the command line (`:`) and enter `importopml path/to/your/feeds.opml` and press enter.

**Hint**: You can also export an OPML file via `exportopml path/to/your/feeds.opml`

---

## Learning the Interface

eilmeldung has three main panels:

1. **Feed List** (left): Shows your feeds, categories, tags, and custom queries (this is *customizable*!)
2. **Article List** (top right): Displays articles from the selected feed/tag/query
3. **Article Content** (bottom right): Shows the full article content

### Basic Navigation

- `h` / `l`: Move between panels (left/right)
- `j` / `k`: Move up/down within a panel
- `Tab`: Cycle forward through panels
- `g f` / `g a` / `g c`: Jump directly to feeds / articles / content

### Reading Articles

- `o`: Open article in browser, mark as read, jump to next unread
- `x`: Scrape full article content (when preview is truncated)
- `z`: Toggle zen mode (hide everything except article content)

### Managing Article Status

- `r` / `u`: Mark as read / unread
- `m` / `v`: Mark (star) / unmark article
- `t`: Tag an article (press Tab for suggestions)

### Searching and Filtering

- `/`: Search articles
- `=`: Filter articles by query
- See [Article Queries](queries.md) for powerful query syntax

---

## Next Steps

Now that you're set up, explore these topics:

- **[Key Bindings](keybindings.md)** - Complete reference of all shortcuts
- **[Article Queries](queries.md)** - Learn to search and filter articles powerfully
- **[Commands](commands.md)** - Discover all available commands
- **[Configuration](configuration.md)** - Customize appearance, behavior, and key bindings
- **[FAQ](faq.md)** - Common questions and answers

**Quick tips:**
- Press `s` regularly to sync your feeds
- Use `1`, `2`, `3` to quickly switch between all/unread/marked article views
- Create tags and custom queries in your feed list for quick access to important articles
- Customize your workflow with [custom key bindings](configuration.md#input-configuration)
