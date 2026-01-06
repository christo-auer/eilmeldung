## Article Queries

**eilmeldung** features a flexible query language that can be used across multiple contexts throughout the application. Queries are useful to display specific articles or execute bulk-operations. 

---

## Table of Contents

- [Quick Examples](#quick-examples)
- [Query Keys](#query-keys)
- [Search Term Types](#search-term-types)
- [Advanced Features](#advanced-features)
- [Example Queries](#example-queries)
- [Using Sort Orders in Queries](#using-sort-orders-in-queries)
- [Query Cookbook](#query-cookbook)

---

## Quick Examples

Here are some examples to get you started:

```
tag paywall title:/Anzeige:/ feed:heise   # tags all articles from heise whose title starts with "Anzeige" with tag paywall
read #paywall                         # marks all articles with the tag #paywall is read
filter newer:"1 hour ago" unread          # show only unread articles which are newer than one hour
read unread feed:pitchfork summary:rap    # marks all unread articles from pitchfork which contain rap (Case-insensitive) as read
```

Query elements are **conjunctive** (AND-ed together), i.e., all specified conditions must be met for an article to match. To create more complex filters, use negation (`~`) or regular expressions with OR operators in case of search terms.


## Query Keys

| Key               | Syntax                     | Description                                                         | Example                     |
| -----             | --------                   | -------------                                                       | ---------                   |
| `read`            | `read`                     | Match read articles                                                 | `read`                      |
| `unread`          | `unread`                   | Match unread articles                                               | `unread`                    |
| `marked`          | `marked`                   | Match marked articles                                               | `marked`                    |
| `unmarked`        | `unmarked`                 | Match unmarked articles                                             | `unmarked`                  |
| `tagged`          | `tagged`                   | Match articles with at least one tag                                | `tagged`                    |
| `title:`          | `title:<search term>`      | Match articles by title                                             | `title:election`            |
| `summary:`        | `summary:<search term>`    | Match articles by summary/description                               | `summary:"climate change"`  |
| `author:`         | `author:<search term>`     | Match articles by author                                            | `author:smith`              |
| `feed:`           | `feed:<search term>`       | Match articles by feed name                                         | `feed:techcrunch`           |
| `feedurl:`        | `feedurl:<search term>`    | Match articles by feed URL                                          | `feedurl:example.com`       |
| `feedweburl:`     | `feedweburl:<search term>` | Match articles by feed website URL                                  | `feedweburl:github.com`     |
| `all:`            | `all:<search term>`        | Search across all fields (title, summary, author, feed)             | `all:technology`            |
| `tag:`            | `tag:#tag1,#tag2,...`      | Match articles with any of the specified tags                       | `tag:#important,#tech`      |
| `#tag1,#tag2,...` | `#tag1,#tag2,...`          | Same as `tag:#tag1,#tag2,...`                                       | `#important,#tech`          |
| `newer:`          | `newer:"<time>"`           | Match articles newer than specified time                            | `newer:"1 week ago"`        |
| `older:`          | `older:"<time>"`           | Match articles older than specified time                            | `older:"2024-01-01"`        |
| `today`           | `today`                    | Match articles from the last 24 hours                               | `today`                     |
| `syncedbefore:`   | `syncedbefore:"<time>"`    | Match articles synced before specified time                         | `syncedbefore:"1 hour ago"` |
| `syncedafter:`    | `syncedafter:"<time>"`     | Match articles synced after specified time                          | `syncedafter:"2024-12-01"`  |

## Search Term Types

- **Word**: Case-insensitive word match: `title:rust`
- **Quoted String**: Exact phrase match: `title:"Rust programming"`
- **Regular Expression**: Regex pattern match (see [regex syntax documentation](https://docs.rs/regex/latest/regex/#syntax)): `title:/^rust.*guide$/`

## Advanced Features

- **Negation**: Use `~` to negate any query (e.g., `~read` matches unread articles, `~title:politics` excludes articles with "politics" in title)
- **Multiple Criteria**: Combine multiple queries with spaces: all conditions must be satisfied (AND logic)
- **Relative Time**: Use natural language for time-based queries: `"1 week ago"`, `"yesterday"`, `"3 days ago"` (see [`parse_datetime` documentation](https://lib.rs/crates/parse_datetime) for more information)
- **Regular Expression OR**: Use the `|` operator in regex patterns for OR logic: `title:/(rust|python|javascript)/` matches articles with any of these languages in the title

## Example Queries

```
unread today                                    # Unread articles from today
feed:bbc ~read                                  # Unread articles from BBC feed
title:/(?i)breaking|urgent/ newer:"1 hour ago"     # Recent breaking or urgent news (regex OR)
marked #important                               # Marked articles tagged as important
all:"climate change" newer:"1 week ago"         # Climate change articles from last week
~#politics unread                               # Unread articles without politics tag
author:/(?i)smith|jones|brown/                   # Articles by Smith, Jones, or Brown (regex OR)
title:/(feature|bug|fix)/ feed:/github|gitlab/  # Development-related articles from code hosting platforms
```


### Using Sort Orders in Queries

Sort orders can be embedded in queries using the `sort` key. This is particularly useful in feed list queries. Check [Commands](commands.md#sorting-articles) for how sort orders are defined.

**Syntax:**
```
sort:"<sort order>"
```

**Note** the double quotes!

**Examples:**
```
unread sort:"date"                                     # Unread articles, newest first
feed:bbc sort:"feed title"                             # BBC articles sorted by feed then title
today sort:"synced"                                    # Today's articles, most recently synced first
#important unread sort:"<date"                         # Important unread, oldest first
newer:"1 week" sort:"feed date"                        # Last week's articles by feed, newest first
```

### Feed List Query Examples with Sorting

In your `feed_list` configuration, you can add sort orders to queries:

```toml
feed_list = [
  'query: "Latest Unread" unread sort:"date"',
  'query: "By Feed" unread sort:"feed date"',
  'query: "Recently Synced" sort:"synced"',
  "feeds",
  "tags",
]
```

---

## Query Examples

```
# Morning news: unread articles from the last 12 hours
newer:"12 hours ago" unread

# Today's important updates
today unread #important

# Catch up on specific feeds
feed:/tech|news/ unread today

# Quick scan: just headlines from trusted sources
feed:/(bbc|reuters|ap)/ unread newer:"6 hours ago"

# Follow a developing story
all:"climate summit" newer:"1 week ago"

# Compare coverage across sources
title:election newer:"3 days ago"

# Track multiple related topics (case-insensitive)
title:/(?i)(AI|machine learning|neural network)/

# Find related content by same author
author:smith unread
```

## Bulk Operations

Queries can be used for builk operations:

```
# Mark old articles as read
:read older:"2 months ago" unread

# Tag articles from specific feeds
:tag tech feed:/(github|gitlab|stackoverflow)/

# Remove tag from read articles
:untag toread read

# Mark urgent items (case-insensitive)
:mark title:/(?i)breaking|urgent/ newer:"1 hour ago"
```


---

## Tips & Tricks

**Combining Queries with Commands:**
- Use queries with any command that accepts a scope
- Examples: `:read <query>`, `:mark <query>`, `:tag <name> <query>`

**Testing Queries:**
- Use `:filter <query>` to preview results before bulk operations

**Saving Frequent Queries:**
- Add them to your feed list in the configuration file
- Create keybindings for common query-based commands


