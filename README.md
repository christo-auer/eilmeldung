

![Logo of eilmeldung](docs/images/logo.png) 
  

![Screenshot of eilmeldung](docs/images/hero-shot.jpg) 

*eilmeldung* is a *TUI RSS reader* based on the awesome [news-flash](https://gitlab.com/news-flash/news_flash) library.  
- *fast* in every aspect: non-blocking terminal user interface, (neo)vim-inspired keybindings, instant start-up and no clutter
- *stands* on the shoulder of *giants*: based on the news-flash library, *eilmeldung* supports many RSS providers, is efficient and reliable
- *powerful* and yet *easy to use out-of-the-box*: sane defaults which work for most, and yet configurable to meet anyones requirements, from keybindings to colors, from displayed content to RSS provider
- read news like a pro: filter and search news with a easy-to-learn powerful *query language*, activate *zen mode* to focus on the article content and nothing else

*eilmeldung* is German for *breaking news*

# Showreel

https://github.com/user-attachments/assets/a8a1dc60-0705-4521-a88d-3520923d2891

This video demonstrates
- basic (vim-like) navigation and reading
- *zen* mode: just show content
- creating new tags and tagging a article
- *filtering* and *searching* article list by using a article queries
- *tagging* multiple articles by using an article query

# Installation 

Follow any of the installation methods below, then run *eilmeldung*. It will guide you through the setup process.

## Important: Nerd Fonts

You need a [Nerd Font](https://github.com/ryanoasis/nerd-fonts) compatible font/terminal for icons to display correctly!

## Via Homebrew

To install via [homebrew](https://brew.sh), tap this repository and install *eilmeldung*:

```bash
brew tap christo-auer/eilmeldung https://github.com/christo-auer/eilmeldung
brew install eilmeldung
```

## Via AUR (Arch)

There are two AUR packages: `eilmeldung` compiles the latest release and `eilmeldung-git` the `HEAD` of `main`. Use `paru` or `yay` to install.

## Via Cargo

In order to compile `eilmeldung` from source, you need `cargo` with a `rust` compiler with at least edition 2024 (e.g., use `rustup`) and some build deps. On Debian/Unbuntu-based systems its:

```bash
apt update
apt install --yes sudo apt-get install -y build-essential libssl-dev pkg-config libxml2-dev clang libsqlite3-dev

```

Then install *eilmeldung* via:

```
cargo install --git https://github.com/christo-auer/eilmeldung
```


## Nix Flake and Home Manager

<details>
<summary> Expand for installation on Nix and Home Manager</summary>

  Add *eilmeldung* to your inputs, apply `eilmeldung.overlays.default` overlay to `pkgs`. If you want Home Manager integration, add Home Manager module `eilmeldung.homeManager.default`. Here is an example:

  ```nix
  {
    inputs = {
      // ...
      eilmeldung.url = "github:christ-auer/eilmeldung";
    };

    outputs = { nixpkgs, home-manager, eilmeldung, ... }: {
      homeConfigurations."..." = home-manager.lib.homeManagerConfiguration {
        pkgs = import nixpkgs {
          system = "x86_64-linux";
          overlays = [ eilmeldung.overlays.default ];
        };
        
        modules = [
          // ...
          eilmeldung.homeManagerModules.default
        ];
      };
    };
  }
  ```

Home Manager configuration works by defining the settings from the configuration file:

```nix
programs.eilmeldung = {
  enable = true;

  settings = {
    refresh_fps = 60;
    article_scope = "unread";


    theme = {
      color_palette = {
        background = "#1e1e2e";
        // ...
      };
    };

    input_config.mappings = {
        "q" = "quit";
        "j" = "down";
        "k" = "up";
        "g g" = "gotofirst";
        "G" = "gotolast";
        "o" = ["open" "read" "nextunread"];
    };

    feed_list = [
      "query: \"Today Unread\" today unread"
      "query: \"Today Marked\" today marked"
      "feeds"
      "* categories"
      "tags"
    ];
  };
};


```

</details>


# Getting Started

Once installed, run `eilmeldung` to begin. On first launch, you'll be guided through:

## Initial Setup

**Note**: inoreader is currently **NOT** directly supported. Create an issue if you need support for inoreader!

1. Choose Your Provider: Select from local or cloud-based RSS providers (FreshRSS, Miniflux, Feedbin, Local, etc.)
2. Configure Authentication: Enter your credentials (username/password or token, depending on the provider)
3. Initial Sync: The app will sync your feeds from the provider

If you're new to RSS, select **Local** as your provider - it stores everything on your machine without requiring an external service.

## Help on Key Bindings

Press `?` to see all available key bindings. You can also press `/` in the help dialog to filter for specific commands.

## Command Line

Before we begin: `eilmeldung` has a powerful command line which you can open with `:`. All actions you can trigger in `eilmeldung` are commands and key bindings are just executing one or more commands in the background. Some commands also just open the command line with a predefined command.

**Tip**: Press `Tab` in the command line to trigger autocomplete and see helpful suggestions!

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

- Rename: Select a feed/category and press `c w`, then type the new name
- Move: Press `c y` to yank (copy) a feed, navigate to destination, press `c p` to paste after (or `c P` to paste before)
- Remove: Press `c d` to remove an empty feed/category, or `c x` to remove with all children

### Importing an OPML File

Instead of manually adding feeds and categories, you can also import an OPML file. An OPML file contains all the categories and feeds you have defined in another RSS reader or provider:

- Export an OPML file from your current provider and save the OPML file somewhere in your home directory.
- In `eilmeldung`, open the command line (`:`) and enter `importopml path/to/your/feeds.opml` and press enter.

**Hint**: You can also export an OPML file via `exportopml path/to/your/feeds.opml`

## Essential Key Bindings

**Note**: You can redefine all key bindings according to you likings. You can also add completely new key bindings via the [configuration file](docs/configuration.md).


### Syncing & Refreshing

| Key | Action |
|-----|--------|
| `s` | Sync all feeds with your RSS provider |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Move down / up (vim-style) |
| `h` / `l` | Move left (article → article list → feeds) / right (feeds → article list → article) |
| `gg` | Go to first item |
| `G` | Go to last item |
| `space` | Toggle category tree node (open/close) |
| `C-h`/`C-l` | Navigate up/down in feed tree |
| `Ctrl-f` / `Ctrl-b` | Page down / page up |
| `Tab` / `Shift-Tab` | Cycle through panels (forward / backward) |
| `g f` / `g a` / `g c` | Jump directly to feeds / articles / content panel |

### Reading Articles

| Key | Action |
|-----|--------|
| `o` | Open article in browser, mark as read, and jump to next unread |
| `O` | Open all unread articles in browser and mark all as read |
| `J` | Mark current article as read and jump to next unread |
| `x` | Scrape full article content from the web (for truncated articles) |

### Read/Unread Status


| Key | Action |
|-----|--------|
| `r` | Mark current article as read |
| `R` | Mark **all** articles as read (asks for confirmation) |
| `u` | Mark current article as unread |
| `U` | Mark **all** articles as unread (asks for confirmation) |
| `Ctrl-r` | Open command line to set read with query (e.g., `:read unread today`) |


**Note**: These commands are context-dependent! In the article list, they act on the *current article* or *all articles* in the list. On the feed list/tree they act on the *current category/feed* or *all categories/feeds*.

### Marking Articles

| Key | Action |
|-----|--------|
| `m` | Mark current article |
| `M` | Mark **all** articles (asks for confirmation) |
| `v` | Unmark current article |
| `V` | Unmark **all** articles (asks for confirmation) |


**Note**: These commands are context-dependent! In the article list, they act on the *current article* or *all articles* in the list. On the feed list/tree they act on the *current category/feed* or *all categories/feeds*.
### Zen Mode

| Key | Action |
|-----|--------|
| `z` | Toggle distraction-free mode (hides all panels except article content) |

### Tags

| Key | Action |
|-----|--------|
| `t` | Open command line to tag article (e.g., `:tag tech`), **TAB** to autocomplete tag names |

You can create new tags with `:tagadd urgent red` (press **TAB** for autocomplete colors!). Once created, you can bulk-tag articles: `:tag tech unread` tags all unread articles as `tech`.

### Article Views

| Key | Action |
|-----|--------|
| `1` | Show all articles |
| `2` | Show only unread articles |
| `3` | Show only marked articles |

### Searching & Filtering

| Key | Action |
|-----|--------|
| `/` | Search articles (type query and press Enter) |
| `n` / `N` | Jump to next / previous match |
| `=` | Open command line to filter articles |
| `+ +` | Apply current filter |
| `+ r` | Clear filter and show all articles |


See below or [Article Queries](docs/queries.md) for how to craft powerful queries.

### Command Line

| Key | Action |
|-----|--------|
| `:` | Open command line for advanced commands |
| `Esc` or `Ctrl-g` | Cancel command input |
| `Ctrl-u` | Clear command input |
| `Tab`, `Backtab` | Trigger/cycle autocomplete and show help |

## Query Basics

**eilmeldung** supports powerful queries for searching and bulk operations:

### Simple Queries

```
unread                          # All unread articles
marked                          # All marked  articles
today                           # Articles from last 24 hours
unread today                    # Unread articles from today
```

### Search by Field

```
title:security                  # Articles with "security" in title
author:john                     # Articles by John
feed:bbc                        # All articles from BBC feed
summary:"climate change"        # Articles with exact phrase in summary
```

### Time-Based

```
newer:"1 week ago"              # Articles from last week
older:"2024-01-01"              # Articles before Jan 1, 2024
newer:"1 hour ago" unread       # Recent unread articles
```

### Tags

```
#important                      # Articles tagged "important"
#tech,#news                     # Articles tagged "tech" OR "news"
~#politics                      # Articles NOT tagged "politics"
```

### Regular Expressions

```
title:/breaking|urgent/         # "breaking" OR "urgent" in title
feed:/github|gitlab/            # Articles from GitHub or GitLab feeds
```

### Bulk Operations with Queries

```
:read unread feed:bbc           # Mark all unread BBC articles as read
:tag tech newer:"1 day"         # Tag recent articles with "tech"
:mark title:/important/i        # Mark all articles with "important" in title
```

For complete query documentation, see [Article Queries](docs/queries.md).

---

# Documentation

- [Configuration](docs/configuration.md): contains all *configuration options* along with the input configuration
- [Commands](docs/commands.md): *eilmeldung* contains a command line, like (neo)vim, to effectively carry out many operations (e.g., bulk-operations)
- [Article Queries](docs/queries.md): *article queries* can be used to *filter* and *search* according to a multitude of search criteria. Article queries are also supported by bulk-operations (un/tag, un/read, un/mark articles)
- [Command Line Argumnets](docs/cli_args.md): available command line arguments


</details>

# Standing on the Shoulders of Giants

*eilmeldung* was inspired by other awesome programs and libraries of which I want to mention some:

- [news-flash](https://gitlab.com/news-flash/news_flash) library and [news-flash GTK](https://gitlab.com/news-flash/news_flash_gtk), a modern Gnome/GTK RSS reader, both implemented in rust
- [newsboat](https://newsboat.org/) which has been me TUI RSS reader of choice for many years
- [spotify-player](https://github.com/aome510/spotify-player), a TUI spotify music player written in rust. In particular, the theming system and how input is handled has been a great inspiration for *eilmeldung*
- [vifm](https://vifm.info/), [neomutt](https://neomutt.org/) with [notmuch](https://notmuchmail.org/) inspired the filtering and article query systems
- [neovim](https://neovim.io/) and [vim](https://www.vim.org/) for their philosophy on user input
- [ratatui](https://ratatui.rs/) and all its supporting libraries for creating the TUI

# On the use of LLMs in this Project 

<details>

<summary>Expand to learn more about why and how I used LLMs in this project</summary>

This project was built as an experiment in learning Rust through LLM use.

## Some Context

I teach programming/computer science at a university of applied sciences. Over the last few years, I've witnessed a change in how students *learn* and *understand* programming and related concepts by using LLMs. While for some students, using LLMs brings real benefits, for others it becomes a crutch that prevents genuine learning. The difference lies not in the tool itself, but in *how* it's used. I am not only talking about *cheating* in assignments. The main problems are in my opinion:

1. LLMs are trained to produce code and solve problems: when a student encounters a problem, LLMs tend to produce code, preventing students from overcoming the challenge themselves and robbing them of a vital learning opportunity.
2. As LLMs tend to be sycophantic and pleasing in the nature of their answers, students fall into the trap of believing that they understood the concept under investigation. This may be true on a conceptual level. However, programming is a *doing art* which is only understood when students overcome the challenge of *applying a programming concept* (by failing and then succeeding).

Consider this analogy I sometimes use with my students: You want to learn to swim. An LLM can explain the mechanics—how to move your arms, when to breathe, how to stay afloat. But would you then jump into deep water based solely on that explanation? Of course not. You'd need hours of practice in shallow water, struggling, failing, and gradually improving.

Programming is no different. Yet LLMs make it tempting to skip the struggle entirely. To be fair, the same argument applies to any passive learning method (like YouTube videos or classical lectures). However, never has this approach of purely conceptual learning been so alluring as with LLMs.

That said, LLMs, it seems at the moment, are here to stay. Knowing how to use them (and when not) is a vital ability which already plays a certain role in programming. For this reason I am incorporating "developing using LLMs" into my programming course ("Advanced topics in Java"). In order to make sure to really understand what I am talking about, I needed to apply LLMs to *learn a new programming language* myself. And this project *eilmeldung* is the result of this endeavour.

## How LLMs were used in this Project

LLMs were used in this project to understand if and how they can be used for the following:

- Learning a new programming language or concept using a *Tutor Agent Prompt*: The tutor agent prompt tells the LLM to *not produce any solutions* or *code*. Instead, the LLM was prompted to lead me to a solution by asking questions. This approach was applied also to compiler errors.
- Explaining existing code bases using a *Explainer Agent Prompt*: With this prompt, the LLM explains existing code bases to more quickly understand *idiomatic programming approaches* and *architectures*.
- Creating documentation (e.g., [Commands](docs/commands.md))
- Refactoring after a certain pattern: After refactoring one or more modules, the LLM was asked to refactor remaining modules in a similar manner.
- Creating fine-grained commits.

## How LLMs were NOT used in this Project

This project is **not vibe-coded**. Every line was intentionally written to solve a problem I understood. The code has *warts*, i.e., awkward Rust patterns, over-engineered solutions, remnants of learning mistakes --- and that's the point. This is what *learning* looks like.

## Purely Anecdotal Lessons Learned

- Using an LLM as a tutor was mostly successful. For example, when debugging borrow checker errors, having the LLM *ask* me questions like "What is the lifetime of this reference?" was far more educational than receiving a corrected code snippet. However, as the context becomes longer, LLMs tend to forget their role as tutors (*context rot*) and start to produce code again. Apart from that, LLMs *can* be really good sparring partners when it comes to learning.
- Explaining unknown code bases works relatively well as long as the code base is not too large and questions are either very specific or very high-level.
- Creating documentation works but needs to be checked *very carefully* for errors and wrong assumptions.
- Refactoring (in my case) didn't work and I had to revert the changes: LLMs tend to produce code which is not very maintainable and does not fit to the existing architecture.
- Committing: worked well at first but led to data loss in one case (LLM stashed all changes, then dropped the stash and then tried to re-implement the changes itself)

## Key Takeaway 

If you're learning with LLMs:
- **Do**: Use them as tutors that ask questions, not answer machines
- **Do**: Implement solutions yourself, even when LLMs offer code
- **Don't**: Let LLMs rob you of the struggle --- the struggle *is* the learning
- **Don't**: Mistake understanding an explanation for having the skill

## Tools and Prompts

I am using [neovim](https://neovim.io/) with [opencode](https://opencode.ai/). Here are the prompts in *opencode agent format* I've developed for the different tasks:

- [tutor.md](https://github.com/user-attachments/files/24354660/tutor.md): Tutor helping to understand new concepts
- [explainer.md](https://github.com/user-attachments/files/24354664/explainer.md): For understanding code bases
- [unit-tester.md](https://github.com/user-attachments/files/24354665/unit-tester.md): For creating unit test. Note how the LLM should **deny** creating tests on implementations or unclear specification.



</details>
