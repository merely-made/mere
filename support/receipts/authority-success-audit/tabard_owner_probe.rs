use tabard::theme::registry::{THEME_ID_LIGHT, ThemeRegistry, ThemeSource};
use tabard::workshop::{ThemeDraft, WorkshopError};
use tabard::{FontFamily, FontPreferences, FontRole, ThemeFontStyle};

#[test]
fn public_registry_can_replace_a_builtin_that_workshop_refuses() {
    let mut registry = ThemeRegistry::default();
    let original = registry.theme_def(THEME_ID_LIGHT).unwrap().clone();
    let mut replacement = original.clone();
    replacement.name = "Audit replacement".into();
    replacement.source = ThemeSource::User;
    assert!(matches!(ThemeDraft::from_theme(replacement.clone()), Err(WorkshopError::IdentityInUse)));
    assert!(!registry.rename_user_theme(THEME_ID_LIGHT, "Refused"));
    assert!(!registry.remove_user_theme(THEME_ID_LIGHT));
    registry.add_user_theme(replacement).unwrap();
    assert_eq!(registry.theme_def(THEME_ID_LIGHT).unwrap().name, "Audit replacement");
    assert_eq!(registry.theme_def(THEME_ID_LIGHT).unwrap().source, ThemeSource::User);
    assert!(registry.rename_user_theme(THEME_ID_LIGHT, "Now renamed"));
    assert!(registry.remove_user_theme(THEME_ID_LIGHT));
    assert!(registry.resolve_theme(Some(THEME_ID_LIGHT)).fallback_used);
    let mut control = ThemeRegistry::default();
    let fork = control.fork(THEME_ID_LIGHT, "user:audit", "Copy").unwrap();
    assert_eq!(fork.source, ThemeSource::User);
    assert_eq!(control.theme_def(THEME_ID_LIGHT), Some(&original));
    println!("workshop rejects built-in collision; raw registry replaces, renames and removes it; fresh-ID fork preserves original");
}

#[test]
fn generic_font_case_rules_disagree_across_shared_preference_paths() {
    let defaults = FontRole::ALL.into_iter().map(|role| (role, FontFamily::new("sans-serif").unwrap())).collect();
    let upper: FontPreferences = serde_json::from_str(r#"{"reading":["SERIF"]}"#).unwrap();
    let lower: FontPreferences = serde_json::from_str(r#"{"reading":["serif"]}"#).unwrap();
    let resolved_upper = upper.resolve(&[], &defaults);
    assert_eq!(resolved_upper[0].requested[0].name(), "SERIF");
    assert_eq!(resolved_upper[0].family.name(), "sans-serif");
    assert!(resolved_upper[0].used_default);
    assert_eq!(lower.resolve(&[], &defaults)[0].family.name(), "serif");
    assert!(!lower.resolve(&[], &defaults)[0].used_default);
    let authored = ThemeFontStyle { family: Some("SERIF".into()), ..Default::default() };
    authored.validate().unwrap();
    assert_eq!(authored.css_family(FontRole::Reading).as_deref(), Some("serif"));
    assert!(upper.css_custom_properties(&[], &defaults).contains("--tabard-font-reading: sans-serif;"));
    assert!(lower.css_custom_properties(&[], &defaults).contains("--tabard-font-reading: serif;"));
    println!("application SERIF resolves sans-serif fallback; application serif and authored SERIF retain generic serif; requested spelling preserved");
}
