This is still **WIP**!

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

</details>
