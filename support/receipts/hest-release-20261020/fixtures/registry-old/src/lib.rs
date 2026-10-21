/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Prepublication compatibility with the immutable Cambium 0.4.0 package.

#[cfg(test)]
mod tests {
    use command_menu::{Command, CommandChoices, CommandSet, MenuSession, catalogue};

    #[test]
    fn old_and_new_imports_cross_published_cambium_without_conversion() {
        // Old callers can still construct the published all-public struct.
        let old = Command {
            id: catalogue::ids::VIEW_FIT.into(),
            label: "Fit this host".into(),
            category: "view".into(),
            shortcut: Some("Ctrl+0".into()),
            disabled_reason: Some("No content".into()),
        };
        let new: hest::menu::Command = old;
        let published: cambium::Command = new;
        let row = cambium::CommandItem::from(&published);
        assert_eq!(row.id, catalogue::ids::VIEW_FIT);
        assert_eq!(row.shortcut.as_deref(), Some("Ctrl+0"));
        assert_eq!(row.disabled_reason.as_deref(), Some("No content"));

        let mut old_set = CommandSet::new();
        old_set.register(published);
        let new_set: hest::menu::CommandSet = old_set;
        let published_set: cambium::CommandSet = new_set;
        let old_session = MenuSession::open(Some("view"));
        let new_session: hest::menu::MenuSession = old_session;
        let published_session: cambium::MenuSession = new_session;
        let old_choices = CommandChoices::default();
        let new_choices: hest::menu::CommandChoices = old_choices;
        let published_choices: cambium::CommandChoices = new_choices;
        assert_eq!(
            published_session.rows(&published_set, &published_choices)[0].id,
            catalogue::ids::VIEW_FIT
        );
    }

    #[test]
    fn the_old_catalogue_path_reexports_the_same_entries_and_functions() {
        let old: &[catalogue::Entry] = hest::catalogue::CATALOGUE;
        let new: &[hest::catalogue::Entry] = catalogue::CATALOGUE;
        assert!(std::ptr::eq(old, new));
        assert_eq!(catalogue::namespace(catalogue::ids::NODE_NEW), "node");
        assert!(catalogue::is_well_formed(catalogue::ids::RELATION_ADD));
        let command: hest::menu::Command = catalogue::command(catalogue::ids::NODE_NEW).unwrap();
        assert_eq!(Some(command.label.as_str()), catalogue::label(&command.id));
    }

    #[test]
    fn stored_choices_keep_the_published_field_names_and_defaults() {
        let stored = r#"{"added":["node:new"],"removed":["view:fit"],"recent":["session:save"]}"#;
        let old: CommandChoices = serde_json::from_str(stored).unwrap();
        let new: hest::menu::CommandChoices = old;
        let published: cambium::CommandChoices = new;
        assert_eq!(serde_json::to_string(&published).unwrap(), stored);
        let sparse: cambium::CommandChoices =
            serde_json::from_str(r#"{"recent":["view:fit"]}"#).unwrap();
        assert!(sparse.added.is_empty() && sparse.removed.is_empty());
        assert_eq!(sparse.recent, ["view:fit"]);
    }
}
