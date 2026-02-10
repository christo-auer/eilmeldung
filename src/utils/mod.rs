use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

pub mod prelude {
    #[cfg(unix)]
    pub use super::StderrRedirect;
    pub use super::html_sanitize;
    pub use super::patch_text_style;
    pub use super::to_bubble;
}

pub fn html_sanitize(html_escaped_string: &str) -> String {
    htmlescape::decode_html(&html_escaped_string.replace("＆", "&"))
        .unwrap_or(html_escaped_string.to_owned())
}

pub fn to_bubble<'a>(span: Span<'a>) -> Line<'a> {
    let style = span.style;

    Line::from(vec![
        Span::styled("", style),
        span.style(style.add_modifier(Modifier::REVERSED)),
        Span::styled("", style),
    ])
}

/// A RAII guard that redirects stderr to /dev/null for the duration of its lifetime.
/// On Unix systems, this temporarily suppresses stderr output from C libraries.
/// Stderr is automatically restored when the guard is dropped.
#[cfg(unix)]
pub struct StderrRedirect {
    saved_stderr: RawFd,
}

#[cfg(unix)]
impl StderrRedirect {
    /// Create a new stderr redirect, saving the current stderr and redirecting it to /dev/null.
    /// Returns None if the redirection fails.
    pub fn new() -> Option<Self> {
        use std::fs::OpenOptions;

        unsafe {
            // Save the current stderr file descriptor
            let saved_stderr = libc::dup(2);
            if saved_stderr == -1 {
                log::warn!("Failed to duplicate stderr file descriptor");
                return None;
            }

            // Open /dev/null for writing
            let devnull = OpenOptions::new().write(true).open("/dev/null").ok()?;

            // Redirect stderr to /dev/null
            if libc::dup2(devnull.as_raw_fd(), 2) == -1 {
                log::warn!("Failed to redirect stderr to /dev/null");
                libc::close(saved_stderr);
                return None;
            }

            Some(Self { saved_stderr })
        }
    }
}

#[cfg(unix)]
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
