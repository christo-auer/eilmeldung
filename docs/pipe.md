# Piping Article through an External Command

You can pipe the (scraped) article content through an external command to perform all kinds of tasks:

- storing the markdown in a file
- using a custom external scraping mechanism
- filtering the content, e.g., stripping away unwanted tags/ads/etc.
- sending the content of an article via mail to someone else
- piping the content through an LLM
- ...

## Pipe Command

The pipe command takes three arguments: `pipe <in> <out> <command>`

- `<in>`/`<out>` can take the values `md` (Markdown), `html` or `null`
  - `<in>` is the type of content the command receives from `stdin`; `null` means no input
  - `<out>` is the expected content the command produces on `stdout`; `null` means the output is discarded
- `<command>` is a shell command
  - it supports the placeholders `{title}`, `{feed}`, `{date}`, `{author}`, `{url}` which are replaced before the command is executed; if you need other placeholders, please raise an issue
  - it supports quotes like shells (`""` and `''`)
  - environment variable substitution (via `${VAR}` or `$VAR`) and `~` at the beginning are supported
  - any other shell features like `stdin`/`stdout` redirection via pipes (`>`, `<`, `|` and friends) etc. are **not supported**

Some content on how the command is executed:

- The execution of the command is carried out in the background.
- After writing the content to the command, `stdin` is closed.
- Be careful if the command is *blocking* as this will also block any subsequent background operations in `eilmeldung`. There is currently **no timeout mechanism** to abort a command when executing the command through `pipe`.

Here are some examples to understand the syntax:

```
pipe md md sed 's/Java/rust/g'                  # pipes the content through and replaces all occurrences of Java by rust
pipe html null neomutt -e "set content_type=text/html" -s "{title}" -- you@eilmeldung.org 
                                                # sends the HTML content (as an HTML mail) via neomutt
pipe md null ~/.local/save-article.sh "{title}" # passes the markdown to a custom shell script for saving the article in a file
```

## Output

### Processed Output

If `<out>` is `md` or `html`, the result will be displayed as the new article content with a small icon indicating that the content has been processed:

<image here>

The filtered output is *transient*, i.e., if you select a different article and select the formerly processed article again, the normal content will be shown.

### Errors and stderr

An error return code or any content written to `stderr` is shown in the content panel:

<image here>

## Recipes

### Saving Article as Markdown Files

To save the markdown content of an article, you can use the following shellscript (e.g., in `~/.local/bin/save-article.sh`).

```sh
#!/bin/sh
destination="${HOME}/articles"

feed=$(echo "$1" | sed 's%/%-%g') # substitute / by -
date=$(echo "$2" | sed 's%/%-%g') 
title=$(echo "$3" | sed 's%/%-%g')
dir="${destination}/${feed}"
path="${dir}/${date}-${title}.md"

mkdir -p "${dir}"

cat > "${path}"
```

The shell script expects the parameters `{feed}`, `{date}` and `{title}` and stores the article's markdown in `~/articles/{feed}/{date}-{title}.md` with the following `pipe` command:
```
pipe md null ~/.local/bin/save-article.sh "{feed}" "{date}" "{title}"
```

Note that the output is discarded (unless there is an error). For quick execution you should define a key mapping, e.g., to `Ctrl-s`:

```toml
[input_config.mappings]
"C-s"         = ["pipe md null ~/.local/bin/save-article.sh \"{feed}\" \"{date}\" \"{title}\""]
```

### Send Content via neomutt

Define the following key mapping:

```toml
[input_config.mappings]
"Z" = ["cmd pipe html null neomutt -e \"set content_type=text/html\" -s \"{title}\" -- "] 
```

Pressing `Z` opens the command line and you can enter a mail address (or more than one).


### Filtering an Article through an LLM via opencode

`eilmeldung` itself has no LLM features. However, using `pipe` you can process an article's content via an LLM, opencode in this case. You can use the following shellscript for this:

```sh
#!/bin/sh

if [ $# -gt 0 ]; then
  prepend="$@: "
else 
  prepend="summarize this in at most 5 bullet points: "
fi

{ echo "$prepend"; cat; } | opencode run
```

With `pipe md md path-to-shellscript.sh`, the content is summarized by the LLM defined in opencode. You can append a question to ask the LLM something different, e.g., `pipe md md path-to-shellscript.sh is the new album any good?`.

**Note**: The answer from the LLM may take a while (see spinning indicator at the bottom right). If you select a different article while the process is running, the output is discarded!

You can also define keybindings for quick action:

```toml
A = ["pipe md md path-to-shellscript.sh"]
a = ["cmd pipe md md path-to-shellscript.sh"] #opens the command line for entering a custom query

```

