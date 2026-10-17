//! Actual complete navigation/suggestion/mailbox modules with retained imports.
#[path = "../../../crates/shell/chrome/src/nav.rs"]
mod nav;
#[path = "../../../crates/shell/chrome/src/omnibar.rs"]
mod omnibar;
#[path = "../../../crates/shell/chrome/src/suggest.rs"]
mod suggest;

#[test]
fn audit_explicit_search_provider_is_replaced_by_duckduckgo() {
    use omnibar::{OmnibarMatch, SearchProviderKind};
    for provider in [SearchProviderKind::Bing, SearchProviderKind::Google, SearchProviderKind::DuckDuckGo] {
        let selected = OmnibarMatch::SearchQuery { query: "chosen provider".into(), provider };
        assert_eq!(suggest::resolve_match(&selected).as_deref(), Some("https://duckduckgo.com/?q=chosen+provider"));
    }
}

#[test]
fn audit_configured_command_classification_and_suggestions_disagree() {
    assert!(matches!(nav::classify_with(":home", ':'), nav::NavTarget::Command(_)));
    let rows = suggest::suggestions(":home", &nav::History::new("mere://welcome"));
    assert!(matches!(rows.last(), Some(omnibar::OmnibarMatch::SearchQuery { query, .. }) if query == ":home"));
    assert!(matches!(nav::classify_with(">home", ':'), nav::NavTarget::Search(_)));
    assert!(suggest::suggestions(">home", &nav::History::new("mere://welcome")).is_empty());
}
