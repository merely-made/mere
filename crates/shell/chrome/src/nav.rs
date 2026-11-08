// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Chrome's host-neutral navigation model: location-bar text classification,
//! plus the linear back/forward [`History`], which lives in `backstep` beside
//! the editor's other ephemeral histories and is re-exported here.
//!
//! The reused `chrome` domain projects `can_go_back` / `can_go_forward`
//! from a content viewer's *own* history (servo's, in graphshell). meerkat
//! has no such viewer, so it owns this small linear history directly and
//! feeds those flags back into the reused [`ToolbarState`](chrome::toolbar::ToolbarState).
//!
//! URL handling here is deliberately string-light: prefix a scheme onto a
//! bare host, route everything unrecognizable to search. The content engine
//! (the content-root slice) canonicalizes the resolved URL on load — meerkat
//! does not try to be a URL parser.

pub use backstep::nav::History;

/// The default omnibar command-mode sigil. A leading `>` switches the bar from
/// address resolution to the command shell. Configurable (the host passes the
/// user's setting to [`classify_with`]); `>` is the command-palette convention.
pub const DEFAULT_COMMAND_SIGIL: char = '>';

/// What a piece of typed location-bar text resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTarget {
    /// Text already usable as a URL — it carried a scheme, or looked like a
    /// bare host that we prefix with `https://`.
    Url(String),
    /// Free text with no usable scheme or host: a web search.
    Search(String),
    /// A command-shell expression (the text after the command sigil, trimmed).
    /// The host routes this to the `CommandShell` rather than navigating; it
    /// never goes through [`resolve`](Self::resolve).
    Command(String),
}

impl NavTarget {
    /// The absolute URL to actually load. [`Url`](Self::Url) passes through;
    /// [`Search`](Self::Search) becomes a provider query URL (DuckDuckGo for
    /// now — the provider becomes configurable when the omnibar session lands).
    ///
    /// [`Command`](Self::Command) has no URL: the caller branches on it before
    /// resolving, so this returns the raw expression as a harmless total
    /// fallback.
    pub fn resolve(&self) -> String {
        match self {
            NavTarget::Url(u) => u.clone(),
            NavTarget::Search(q) => format!("https://duckduckgo.com/?q={}", encode_query(q)),
            NavTarget::Command(src) => src.clone(),
        }
    }
}

/// Classify location-bar text into a [`NavTarget`] using the default command
/// sigil ([`DEFAULT_COMMAND_SIGIL`]). See [`classify_with`].
pub fn classify(input: &str) -> NavTarget {
    classify_with(input, DEFAULT_COMMAND_SIGIL)
}

/// Resolve a link `href` (from a clicked content-card link) against the document's
/// `base` URL. An absolute href (one that parses standalone, e.g. `gemini://…`) is
/// used as-is; a relative href (`page.gmi`, `../up`, `/root`) joins the base via the
/// `url` crate — the common case for gemtext / markdown links; if neither parses,
/// fall back to the omnibar heuristic ([`classify`]). (Inline-link nav.)
pub fn resolve_href(base: &str, href: &str) -> String {
    if let Ok(absolute) = url::Url::parse(href) {
        return absolute.to_string();
    }
    if let Ok(joined) = url::Url::parse(base).and_then(|b| b.join(href)) {
        return joined.to_string();
    }
    classify(href).resolve()
}

/// Classify location-bar text into a [`NavTarget`], with `sigil` selecting
/// command mode.
///
/// - Leading / trailing whitespace is trimmed.
/// - Text whose first non-whitespace character is `sigil` is a
///   [`Command`](NavTarget::Command); the expression is the remainder, trimmed.
/// - Anything containing `://`, or an authority-less scheme we recognize
///   (`about:`, `data:`, `mailto:`), is taken as a URL verbatim.
/// - Otherwise a single whitespace-free token whose authority is `localhost`
///   or carries a dotted host (`example.com`, `sub.example.co.uk/path`) gets
///   an `https://` prefix.
/// - Everything else is a search query.
pub fn classify_with(input: &str, sigil: char) -> NavTarget {
    let s = input.trim();
    if let Some(expr) = s.strip_prefix(sigil) {
        return NavTarget::Command(expr.trim().to_string());
    }
    if s.is_empty() {
        return NavTarget::Search(String::new());
    }
    if is_verbatim_url(s) {
        return NavTarget::Url(s.to_string());
    }
    if looks_like_host(s) {
        return NavTarget::Url(format!("https://{s}"));
    }
    NavTarget::Search(s.to_string())
}

/// URLs we pass through untouched: anything with an explicit authority
/// (`scheme://…`) and the authority-less schemes a user might type directly.
fn is_verbatim_url(s: &str) -> bool {
    s.contains("://")
        || s.starts_with("about:")
        || s.starts_with("data:")
        || s.starts_with("mailto:")
}

/// Heuristic "this is a host, not a search": a single whitespace-free token
/// whose authority (text before the first `/`, host part before any `:port`)
/// is `localhost` or a dotted name with non-empty labels.
fn looks_like_host(s: &str) -> bool {
    if s.split_whitespace().count() != 1 {
        return false;
    }
    let authority = s.split('/').next().unwrap_or(s);
    let host = authority.split(':').next().unwrap_or(authority);
    host == "localhost" || (host.contains('.') && !host.starts_with('.') && !host.ends_with('.'))
}

/// Percent-encode a search query's bytes, keeping the RFC 3986 unreserved set
/// (`ALPHA` / `DIGIT` / `-` `.` `_` `~`) and mapping spaces to `+` (the form
/// convention every search provider accepts).
fn encode_query(q: &str) -> String {
    let mut out = String::with_capacity(q.len());
    for b in q.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            },
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_passes_through_scheme_urls() {
        assert_eq!(
            classify("https://example.com/x"),
            NavTarget::Url("https://example.com/x".into())
        );
        assert_eq!(
            classify("mere://welcome"),
            NavTarget::Url("mere://welcome".into())
        );
        assert_eq!(
            classify("about:blank"),
            NavTarget::Url("about:blank".into())
        );
    }

    #[test]
    fn resolve_href_passes_through_absolute_and_joins_relative() {
        // An absolute href (any scheme with an authority) is used verbatim.
        assert_eq!(
            resolve_href("gemini://example.org/dir/page.gmi", "gemini://other.test/x"),
            "gemini://other.test/x"
        );
        // A relative href joins the document's base (the common gemtext case).
        assert_eq!(
            resolve_href("gemini://example.org/dir/page.gmi", "next.gmi"),
            "gemini://example.org/dir/next.gmi"
        );
        // A root-relative href joins the base authority.
        assert_eq!(
            resolve_href("gemini://example.org/dir/page.gmi", "/top.gmi"),
            "gemini://example.org/top.gmi"
        );
        // A parent-relative href resolves up the path.
        assert_eq!(
            resolve_href("gemini://example.org/a/b/page.gmi", "../up.gmi"),
            "gemini://example.org/a/up.gmi"
        );
    }

    #[test]
    fn classify_prefixes_bare_hosts() {
        assert_eq!(
            classify("example.com"),
            NavTarget::Url("https://example.com".into())
        );
        assert_eq!(
            classify("sub.example.co.uk/path?q=1"),
            NavTarget::Url("https://sub.example.co.uk/path?q=1".into())
        );
        assert_eq!(
            classify("localhost:8080"),
            NavTarget::Url("https://localhost:8080".into())
        );
    }

    #[test]
    fn classify_routes_sigil_prefix_to_command() {
        assert_eq!(classify(">back"), NavTarget::Command("back".into()));
        // The expression is the remainder, trimmed on both sides.
        assert_eq!(
            classify("  >  for n in nodes() { delete(n) }  "),
            NavTarget::Command("for n in nodes() { delete(n) }".into())
        );
        // A bare sigil is an empty command, not a search.
        assert_eq!(classify(">"), NavTarget::Command(String::new()));
    }

    #[test]
    fn classify_with_honors_a_custom_sigil() {
        assert_eq!(
            classify_with(":home", ':'),
            NavTarget::Command("home".into())
        );
        // Under a `:` sigil, a leading `>` is just ordinary text (a search).
        assert_eq!(
            classify_with(">home", ':'),
            NavTarget::Search(">home".into())
        );
    }

    #[test]
    fn classify_treats_free_text_as_search() {
        assert_eq!(
            classify("hello world"),
            NavTarget::Search("hello world".into())
        );
        assert_eq!(classify("rustlang"), NavTarget::Search("rustlang".into()));
        // Whitespace anywhere disqualifies the host heuristic.
        assert_eq!(
            classify("example .com"),
            NavTarget::Search("example .com".into())
        );
    }

    #[test]
    fn search_resolves_to_encoded_provider_query() {
        let url = NavTarget::Search("a b&c".into()).resolve();
        assert_eq!(url, "https://duckduckgo.com/?q=a+b%26c");
    }

    #[test]
    fn url_resolves_to_itself() {
        assert_eq!(
            NavTarget::Url("https://example.com".into()).resolve(),
            "https://example.com"
        );
    }
}
