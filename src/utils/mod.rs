use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use crate::prelude::*;

pub mod prelude {
    pub use super::StderrRedirect;
    pub use super::html_sanitize;
    pub use super::lex_ordering;
    pub use super::patch_text_style;
    pub use super::to_bubble;
}

pub fn html_sanitize(html_escaped_string: &str) -> String {
    htmlescape::decode_html(&html_escaped_string.replace("＆", "&"))
        .unwrap_or(html_escaped_string.to_owned())
}

pub fn to_bubble<'a>(span: Span<'a>, config: &Config) -> Line<'a> {
    let style = span.style;
    let left_icon = config.icon_set.big_icon_left_icon();
    let right_icon = config.icon_set.big_icon_right_icon();

    Line::from(vec![
        Span::styled(
            left_icon.to_string(),
            if left_icon != ' ' {
                style
            } else {
                style.reversed()
            },
        ),
        span.style(style.reversed()),
        Span::styled(
            right_icon.to_string(),
            if right_icon != ' ' {
                style
            } else {
                style.reversed()
            },
        ),
    ])
}

/// A RAII guard that redirects stderr to /dev/null or NUL for the duration of its lifetime. this
/// temporarily suppresses stderr output from C libraries. Stderr is automatically restored when the
/// guard is dropped.
pub struct StderrRedirect {
    saved_stderr: libc::c_int,
}

impl StderrRedirect {
    /// Create a new stderr redirect, saving the current stderr and redirecting it to /dev/null.
    /// Returns None if the redirection fails.
    pub fn new() -> Option<Self> {
        unsafe {
            // Save the current stderr file descriptor
            let saved_stderr = libc::dup(2);
            if saved_stderr == -1 {
                log::warn!("Failed to duplicate stderr file descriptor");
                return None;
            }

            // Open the null device — /dev/null on Unix, NUL on Windows — using
            // libc::open on both platforms to get a CRT-level fd directly.
            let null_path = if cfg!(windows) { c"NUL" } else { c"/dev/null" };
            let devnull_fd = libc::open(null_path.as_ptr(), libc::O_WRONLY);

            if devnull_fd == -1 {
                log::warn!("Failed to open null device");
                libc::close(saved_stderr);
                return None;
            }

            // Redirect stderr to the null device
            let result = libc::dup2(devnull_fd, 2);
            // Close our copy — fd 2 now independently refers to the null device
            libc::close(devnull_fd);

            if result == -1 {
                log::warn!("Failed to redirect stderr to null device");
                libc::close(saved_stderr);
                return None;
            }

            Some(Self { saved_stderr })
        }
    }
}

impl Drop for StderrRedirect {
    fn drop(&mut self) {
        unsafe {
            // Restore the original stderr
            if libc::dup2(self.saved_stderr, 2) == -1 {
                // Not much we can do here since we can't write to stderr
                log::error!("Failed to restore stderr file descriptor");
            }
            libc::close(self.saved_stderr);
        }
    }
}

pub fn patch_text_style(text: &mut Text, style: Style) {
    text.iter_mut()
        .flat_map(Line::iter_mut)
        .for_each(|span| span.style = span.style.patch(style));
}

struct LexOrdering {
    symbols: Vec<char>,
    current: Vec<usize>,
}

impl LexOrdering {
    pub fn new(symbols: Vec<char>) -> Self {
        LexOrdering {
            symbols,
            current: Default::default(),
        }
    }
}

impl Iterator for LexOrdering {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut index = 0;

        let max_symb = self.symbols.len().saturating_sub(1);

        while let Some(symbol_index) = self.current.get_mut(index)
            && *symbol_index == max_symb
        {
            *symbol_index = 0;
            index += 1;
        }

        match self.current.get_mut(index) {
            Some(entry) => *entry += 1,
            None => self.current.push(0),
        }

        Some(
            self.current
                .iter()
                .rev()
                .filter_map(|index| self.symbols.get(*index))
                .collect::<String>(),
        )
    }
}

pub fn lex_ordering(symbols: Vec<char>) -> Result<impl Iterator<Item = String>, &'static str> {
    if symbols.is_empty() {
        return Err("there must be at least one symbol");
    }

    Ok(LexOrdering::new(symbols))
}

#[cfg(test)]
mod test {
    use claims::assert_some_eq;

    use super::*;
    #[test]
    fn test_lex_ordering() {
        let symbols = vec!['a', 'b', 'c'];
        let mut lex_ordering = lex_ordering(symbols).unwrap();

        vec![
            "a", "b", "c", "aa", "ab", "ac", "ba", "bb", "bc", "ca", "cb", "cc", "aaa", "aab",
            "aac", "aba", "abb", "abc", "aca", "acb", "acc", "baa", "bab", "bac", "bba", "bbb",
            "bbc", "bca", "bcb", "bcc", "caa", "cab", "cac", "cba", "cbb", "cbc", "cca", "ccb",
            "ccc", "aaaa",
        ]
        .iter()
        .for_each(|expected| {
            assert_some_eq!(lex_ordering.next(), expected.to_owned());
        });
    }

    #[test]
    fn test_lex_ordering_no_symbols() {
        if lex_ordering(Default::default()).is_ok() {
            panic!("must return Err if no symboled are passed");
        }
    }
}
