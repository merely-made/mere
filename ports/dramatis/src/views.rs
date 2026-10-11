// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained, accessible public cards. All mutations become custody requests.
use crate::{Request, State};
use cambium::{AnyView, GenetCtx, GenetElement, button, el, text};
use dramatis::view::VaultLockView;
use graphshell::native::quarantine::ReviewAction;
pub type Child = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
pub type Logic = fn(&State) -> Child;
fn line(s: impl Into<String>) -> Child {
    Box::new(el("p", text(s.into())))
}
fn action(label: &str, request: Request, disabled: bool) -> Child {
    let view = button(label.to_string(), move |state: &mut State, _| {
        if !disabled {
            state.queue(request.clone());
        }
    });
    if disabled {
        Box::new(view.attr("disabled", ""))
    } else {
        Box::new(view)
    }
}
pub fn root(state: &State) -> Child {
    let mut body: Vec<Child> = vec![
        Box::new(el("h1", text("Dramatis"))),
        line("Your personae and imported credentials"),
    ];
    if !state.notice.is_empty() {
        body.push(Box::new(
            el("p", text(state.notice.clone())).attr("role", "status"),
        ));
    }
    body.push(action("Reconnect", Request::Refresh, state.busy));
    match &state.snapshot {
        None => body.push(line(
            "Identity pending. Djinn must be running to manage your personae.",
        )),
        Some(snapshot) => {
            let locked = snapshot.lock == VaultLockView::Locked;
            body.push(line(if locked {
                "Vault locked. Unlock with Djinn to choose a persona or review imports."
            } else {
                "Vault unlocked. Djinn holds your credentials and asks for consent."
            }));
            body.push(action(
                if locked {
                    "Unlock with Djinn"
                } else {
                    "Lock vault"
                },
                if locked {
                    Request::Unlock
                } else {
                    Request::Lock
                },
                state.busy,
            ));
            body.push(Box::new(el("h2", text("Personae"))));
            if snapshot.roster.entries.is_empty() {
                body.push(line(
                    "No personae yet. Create one with Djinn's personae-vault tool.",
                ));
            }
            // Reuse the suite's roster row convention; create is outside this slice.
            let rows = persona_picker::roster_items(&snapshot.roster);
            for (profile, row) in snapshot.roster.entries.iter().zip(rows.iter()) {
                let name = &row.label;
                let label = format!("{}{}", name, if profile.chosen { " · in use" } else { "" });
                body.push(action(
                    &label,
                    Request::Select(profile.id.clone()),
                    state.busy || locked || profile.chosen,
                ));
            }
            if !locked && !snapshot.unassigned.is_empty() {
                body.push(Box::new(el("h2", text("Older imports awaiting a persona"))));
                body.push(line("Choose the persona these imports belong to, then ask Djinn to assign them. Assignment preserves each item’s state. Quarantined items still need review."));
                for import in &snapshot.unassigned {
                    let mut contents = vec![line(format!(
                        "{} items · {}",
                        import.item_count, import.persona
                    ))];
                    contents.push(line(format!("Import store: {}", import.store.0)));
                    contents.extend(import.titles.iter().take(8).map(|t| line(t.clone())));
                    contents.push(action(
                        "Assign to this persona",
                        Request::Assign(graphshell::native::quarantine::AssignImportRequest {
                            store: import.store.clone(),
                            profile: snapshot.roster.chosen.clone(),
                            persona: import.persona,
                            revision: import.revision.clone(),
                        }),
                        state.busy,
                    ));
                    body.push(Box::new(el("section", contents).attr("class", "card")));
                }
            }
            body.push(Box::new(el("h2", text("Quarantine review"))));
            if locked {
                body.push(line(
                    "Quarantined items stay sealed while the vault is locked.",
                ));
            } else if snapshot.cards.is_empty() {
                body.push(line("No quarantined imports for this persona."));
            }
            for card in &snapshot.cards {
                let mut contents = vec![Box::new(el("h3", text(card.item.title.clone()))) as Child];
                contents.push(line(format!("Import store: {}", card.store.0)));
                if let Some(subtitle) = &card.item.subtitle {
                    contents.push(line(subtitle.clone()));
                }
                contents.push(line(
                    card.item
                        .credentials
                        .iter()
                        .filter_map(|c| c.kind.cxf_type())
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
                if let Some(reason) = &card.accept_blocked {
                    contents.push(line(reason.clone()));
                } else {
                    contents.push(line(
                        "Imported for review. Djinn checks acceptance and asks for your consent.",
                    ));
                }
                let accept = card.clone();
                let delete = card.clone();
                let blocked = state.busy || locked || card.accept_blocked.is_some();
                let accept_view = button("Accept", move |s: &mut State, _| {
                    if !blocked {
                        s.review(&accept, ReviewAction::Accept);
                    }
                });
                contents.push(if blocked {
                    Box::new(accept_view.attr("disabled", ""))
                } else {
                    Box::new(accept_view)
                });
                let blocked = state.busy || locked;
                let delete_view = button("Delete", move |s: &mut State, _| {
                    if !blocked {
                        s.review(&delete, ReviewAction::Delete);
                    }
                });
                contents.push(if blocked {
                    Box::new(delete_view.attr("disabled", ""))
                } else {
                    Box::new(delete_view)
                });
                body.push(Box::new(el("section", contents).attr("class", "card")));
            }
        },
    }
    Box::new(el("main", body).attr("class", "dramatis"))
}
pub const SHEET: &str = r#"
html { background: #f4f1e9; color: #25251f; font-family: system-ui; }
body { margin: 0; }
main { padding: 24px; max-width: 820px; }
h1 { font-size: 30px; margin: 0 0 8px; }
h2 { font-size: 20px; margin: 24px 0 12px; }
h3 { font-size: 17px; margin: 0 0 8px; }
p { line-height: 1.5; margin: 8px 0; }
button { margin: 6px 10px 6px 0; padding: 10px 16px; border: 1px solid #76766b; border-radius: 6px; background: #fffdf7; color: #25251f; }
button:focus-visible { outline: 3px solid #315e75; outline-offset: 2px; }
button:disabled { opacity: 0.55; }
.card { background: #fffdf7; border: 1px solid #c6c6b6; border-radius: 8px; padding: 16px; margin: 12px 0; }
"#;
