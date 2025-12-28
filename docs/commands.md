## Commands

**eilmeldung** provides a comprehensive command system that can be invoked in two ways:

- **Command line**: Press `:` to open the command line, then type your command and press Enter
- **Key bindings**: Commands are bound to key sequences (see the Keybindings section for defaults and customization)

Commands may accept parameters such as scopes, queries, names, URLs, and colors. On the command line, press TAB to see autocomplete suggestions for available values.

## Application Commands

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `quit` | `quit` | All | Quit eilmeldung |
| `cmd` | `cmd [<content>]` | All | Open command line with optional pre-filled content |
| `redraw` | `redraw` | All | Redraw the screen |
| `confirm` | `confirm <command>` | All | Ask for confirmation before executing command (typically used in key bindings) |
| `LOGOUT` | `LOGOUT NOW` | All | Logout and remove ALL local data (requires `NOW` as confirmation) |
| `nop` | `nop` | All | No operation (useful for unmapping key bindings) |

## Panel Management

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `focus` | `focus <panel>` | All | Focus a specific panel: `feeds`, `articles`, or `content`. Examples: `:focus feeds`, `:focus articles` |
| `next` | `next` | All | Focus next panel (stops at article content) |
| `prev` | `prev` | All | Focus previous panel (stops at feed list) |
| `nextc` | `nextc` | All | Focus next panel (cycles back to feed list) |
| `prevc` | `prevc` | All | Focus previous panel (cycles back to content) |
| `zen` | `zen` | Article Content | Toggle distraction-free mode (hides all panels except article content) |

## Feed List Management

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `sync` | `sync` | Feed List | Sync all feeds |
| `feedadd` | `feedadd <URL> [<name>]` | Feed List | Add a new feed. Examples: `:feedadd https://example.com/feed.xml`, `:feedadd https://news.site/rss "News Site"` |
| `categoryadd` | `categoryadd <name>` | Feed List | Add a new category. Example: `:categoryadd Technology` |
| `tagadd` | `tagadd <name> [<color>]` | Feed List | Add a new tag with optional color (e.g., `red`, `#ff0000`). Press TAB for suggestions. Examples: `:tagadd important red`, `:tagadd tech #0088ff` |
| `rename` | `rename <new name>` | Feed List | Rename the selected feed, category, or tag. Example: `:rename Tech News` |
| `remove` | `remove` | Feed List | Remove the selected item (only works for childless items) |
| `removeall` | `removeall` | Feed List | Remove the selected item with all its children |
| `feedchangeurl` | `feedchangeurl <URL>` | Feed List | Change the URL of the selected feed. Example: `:feedchangeurl https://newurl.com/feed.xml` |
| `tagchangecolor` | `tagchangecolor <color>` | Feed List | Change color of selected tag (e.g., `blue`, `#0000ff`). Press TAB for suggestions. Examples: `:tagchangecolor green`, `:tagchangecolor #ff5500` |
| `toggle` | `toggle` | Feed List | Toggle selected feed or category open/closed in the tree |
| `yank` | `yank` | Feed List | Yank (copy) the selected feed or category for moving |
| `paste` | `paste <position>` | Feed List | Paste the yanked item. Position: `before` or `after`. Examples: `:paste after`, `:paste before` |

## Article List

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `show` | `show <scope>` | Article List | Filter articles by scope: `all`, `unread`, or `marked`. Examples: `:show unread`, `:show marked` |
| `nextunread` | `nextunread` | Article List | Select the next unread article in the list |
| `search` | `search <query>` | Article List | Search for articles matching the query. Example: `:search title:security newer:"1 week"` |
| `searchnext` | `searchnext` | Article List | Jump to the next article matching the current search query |
| `searchprev` | `searchprev` | Article List | Jump to the previous article matching the current search query |
| `filter` | `filter <query>` | Article List | Filter the article list by query. Example: `:filter unread author:john` |
| `filterapply` | `filterapply` | Article List | Apply the current filter |
| `filterclear` | `filterclear` | Article List | Clear the current filter and show all articles |
| `scrape` | `scrape` | Article List, Article Content | Scrape the full article content from the web (for articles with truncated content) |

## Article Actions

These commands support a **scope parameter** to target specific articles:
- `.` or omitted: current article only
- `%`: all articles  
- Any query: all articles matching the query

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `read` | `read [<scope>]` | Feed List, Article List | Mark articles as read. Examples: `:read` (current), `:read %` (all), `:read unread today` (unread from today), `:read feed:bbc` (all from BBC) |
| `unread` | `unread [<scope>]` | Feed List, Article List | Mark articles as unread. Examples: `:unread` (current), `:unread %` (all), `:unread marked` (all marked) |
| `mark` | `mark [<scope>]` | Article List | Mark articles (starred/bookmarked). Examples: `:mark` (current), `:mark %` (all), `:mark unread` (all unread) |
| `unmark` | `unmark [<scope>]` | Article List | Unmark articles. Examples: `:unmark` (current), `:unmark %` (all) |
| `open` | `open [<scope>]` | Article List | Open articles in the web browser. Examples: `:open` (current), `:open marked` (all marked) |
| `tag` | `tag <tag name> [<scope>]` | Article List | Add tag to articles. Examples: `:tag important` (current), `:tag tech unread` (all unread), `:tag news %` (all articles) |
| `untag` | `untag <tag name> [<scope>]` | Article List | Remove tag from articles. Examples: `:untag important` (current), `:untag tech marked` (all marked) |
| `share` | `share <target>` | Article List, Article Content | Share article title and URL. Built-in targets: `clipboard`, `reddit`, `mastodon`, `telegram`, `instapaper`. Custom targets (URL and commands) can be defined in the configuration file. Example: `:share clipboard` |

**Note:** The `read` command also supports an optional **target parameter** to specify which panel's selection to use: `:read <target> <scope>`. Target can be `.` (current panel), `feeds` (feed list selection), or `articles` (article list selection). Examples: `:read feeds %` (all articles in selected feed), `:read articles .` (current article in article list).

## Import/Export

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `importopml` | `importopml <path>` | All | Import feeds from an OPML file. Example: `:importopml feeds.opml` |
| `exportopml` | `exportopml <path>` | All | Export all feeds to an OPML file. Example: `:exportopml backup-feeds.opml` |

## Navigation Commands

These commands are typically used via key bindings rather than the command line.

| Command | Syntax | Context | Description |
|---------|--------|---------|-------------|
| `up` | `up` | All | Navigate up in the current context |
| `down` | `down` | All | Navigate down in the current context |
| `left` | `left` | All | Navigate left in the current context |
| `right` | `right` | All | Navigate right in the current context |
| `pageup` | `pageup` | All | Navigate up by one page |
| `pagedown` | `pagedown` | All | Navigate down by one page |
| `gotofirst` | `gotofirst` | All | Navigate to the first item |
| `gotolast` | `gotolast` | All | Navigate to the last item |
| `gotolast` | `gotolast` | All | Navigate to the last item |

## Input-Related Commands

These commands belong to text input (e.g. command-line or search input) and must be assigned to single keys:

| Command   | Syntax   | Context             | Description              |
| --------- | -------- | ---------           | -------------            |
| `submit`  | `submit` | Input               | Submit the current input |
| `abort`   | `abort`  | Input               | Abort the current input  |
| `clear`   | `clear`  | Input               | Clear the current input  |
| `find`    | `find`   | Input, Article List | Open find (search) input |


## Example Commands

```
:quit                                    # Quit the application
:sync                                    # Sync all feeds
:read                                    # Mark current article as read
:read %                                  # Mark all articles as read
:read unread today                       # Mark all unread articles from today as read
:read feeds %                            # Mark all articles in selected feed as read
:filter title:/breaking/ newer:"1 hour"  # Filter breaking news from last hour
:tag important                           # Tag current article as important
:tag tech unread                         # Tag all unread articles as tech
:untag work %                            # Remove work tag from all articles
:share clipboard                         # Share current article to clipboard
:feedadd https://example.com/feed.xml    # Add a new feed
:feedadd https://news.site/rss News      # Add feed with custom name
:categoryadd Technology                  # Add a new category
:tagadd urgent red                       # Add a red "urgent" tag
:rename Tech News Daily                  # Rename selected item
:importopml feeds.opml                   # Import feeds from OPML file
:exportopml backup.opml                  # Export feeds to OPML file
:focus articles                          # Focus the article list panel
:show unread                             # Show only unread articles
:search author:john newer:"3 days"       # Search for articles by John from last 3 days
```

