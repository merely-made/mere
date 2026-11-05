// SPDX-License-Identifier: Zlib
//! Checked graphical choices. An explicit answer needs both success and a token.
//! Closing a window or failing to start it must not become a remembered No.

use std::fmt;
use std::process::Command;

use crate::{IconType, MessageBoxButton, MessageBoxType};

/// A native message box supplied no usable answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBoxError {
    /// No supported graphical backend could be started.
    Unavailable,
    /// The window closed, was canceled, or the backend refused its invocation.
    NoDecision,
    /// The backend terminated abnormally.
    BackendFailed(Option<i32>),
    /// Successful execution did not supply one of the requested answers.
    InvalidResponse,
}

impl fmt::Display for MessageBoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("native confirmation backend unavailable"),
            Self::NoDecision => f.write_str("native confirmation supplied no decision"),
            Self::BackendFailed(code) => write!(f, "native confirmation failed ({code:?})"),
            Self::InvalidResponse => f.write_str("native confirmation returned an invalid answer"),
        }
    }
}

impl std::error::Error for MessageBoxError {}

#[cfg(target_os = "linux")]
pub(crate) fn show(
    title: &str,
    message: &str,
    buttons: MessageBoxType,
    icon: IconType,
    default: i32,
) -> Result<MessageBoxButton, MessageBoxError> {
    let backend = crate::dialog::DialogBackend::query();
    if !backend.graphic {
        return Err(MessageBoxError::Unavailable);
    }
    show_with_backend(&backend.name, title, message, buttons, icon, default)
}

pub(crate) fn supported(backend: &str) -> bool {
    matches!(
        backend,
        "kdialog" | "zenity" | "matedialog" | "shellementary" | "qarma" | "yad"
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn show_with_backend(
    backend: &str,
    title: &str,
    message: &str,
    buttons: MessageBoxType,
    icon: IconType,
    default: i32,
) -> Result<MessageBoxButton, MessageBoxError> {
    let output = command(backend, title, message, buttons, icon, default)?
        .output()
        .map_err(|_| MessageBoxError::Unavailable)?;
    answer(buttons, output.status.code(), &output.stdout)
}

fn choices(buttons: MessageBoxType) -> &'static [(&'static str, &'static str)] {
    match buttons {
        MessageBoxType::Ok => &[],
        MessageBoxType::OkCancel => &[("cancel", "Cancel"), ("ok", "OK")],
        MessageBoxType::YesNo => &[("no", "No"), ("yes", "Yes")],
        MessageBoxType::YesNoCancel => &[("cancel", "Cancel"), ("no", "No"), ("yes", "Yes")],
    }
}

fn selected(buttons: MessageBoxType, default: i32) -> &'static str {
    match (buttons, default) {
        (MessageBoxType::Ok | MessageBoxType::OkCancel, 1) => "ok",
        (MessageBoxType::YesNo | MessageBoxType::YesNoCancel, 1) => "yes",
        (MessageBoxType::YesNoCancel, 2) | (MessageBoxType::YesNo, _) => "no",
        _ => "cancel",
    }
}

fn command(
    backend: &str,
    title: &str,
    message: &str,
    buttons: MessageBoxType,
    icon: IconType,
    default: i32,
) -> Result<Command, MessageBoxError> {
    if !supported(backend) {
        return Err(MessageBoxError::Unavailable);
    }
    let mut command = Command::new(backend);
    command.arg("--title").arg(title);
    if backend == "kdialog" {
        if buttons == MessageBoxType::Ok {
            command
                .arg(match icon {
                    IconType::Error => "--error",
                    IconType::Warning => "--sorry",
                    _ => "--msgbox",
                })
                .arg(message);
        } else {
            // KDialog supports a default menu entry, not --defaultno. Successful
            // selection emits its fixed tag; parser errors/Cancel emit no answer.
            command
                .arg("--menu")
                .arg(message)
                .arg("--default")
                .arg(selected(buttons, default));
            for (tag, label) in choices(buttons) {
                command.arg(tag).arg(label);
            }
        }
    } else if buttons == MessageBoxType::Ok {
        command
            .arg(match icon {
                IconType::Error => "--error",
                IconType::Warning => "--warning",
                _ => "--info",
            })
            .arg("--text")
            .arg(message)
            .arg("--no-markup");
    } else {
        command.args([
            "--list",
            "--radiolist",
            "--text",
            message,
            "--no-markup",
            "--column",
            "",
            "--column",
            "Choice",
            "--column",
            "Answer",
            "--hide-column=2",
            "--print-column=2",
        ]);
        for (tag, label) in choices(buttons) {
            command.arg(if *tag == selected(buttons, default) {
                "TRUE"
            } else {
                "FALSE"
            });
            command.arg(tag).arg(label);
        }
    }
    Ok(command)
}

fn answer(
    buttons: MessageBoxType,
    code: Option<i32>,
    output: &[u8],
) -> Result<MessageBoxButton, MessageBoxError> {
    match code {
        Some(0) => {},
        Some(1 | 2) => return Err(MessageBoxError::NoDecision),
        _ => return Err(MessageBoxError::BackendFailed(code)),
    }
    let token = std::str::from_utf8(output)
        .map_err(|_| MessageBoxError::InvalidResponse)?
        .trim_end_matches(['\n', '\r']);
    match (buttons, token) {
        (MessageBoxType::Ok, "")
        | (MessageBoxType::OkCancel, "ok")
        | (MessageBoxType::YesNo | MessageBoxType::YesNoCancel, "yes") => Ok(MessageBoxButton::Ok),
        (MessageBoxType::OkCancel | MessageBoxType::YesNoCancel, "cancel")
        | (MessageBoxType::YesNo, "no") => Ok(MessageBoxButton::Cancel),
        (MessageBoxType::YesNoCancel, "no") => Ok(MessageBoxButton::No),
        _ => Err(MessageBoxError::InvalidResponse),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(backend: &str, buttons: MessageBoxType, default: i32) -> Vec<String> {
        command(
            backend,
            "title",
            "message",
            buttons,
            IconType::Question,
            default,
        )
        .unwrap()
        .get_args()
        .map(|s| s.to_string_lossy().into_owned())
        .collect()
    }

    #[test]
    fn negative_default_is_a_supported_selected_choice() {
        let arguments = args("kdialog", MessageBoxType::YesNo, 0);
        assert!(arguments.windows(2).any(|a| a == ["--default", "no"]));
        assert!(
            !arguments
                .iter()
                .any(|a| a == "--defaultno" || a == "--attach")
        );
        let arguments = args("zenity", MessageBoxType::YesNo, 0);
        assert!(arguments.windows(3).any(|a| a == ["TRUE", "no", "No"]));
        assert!(arguments.contains(&"--no-markup".to_string()));
    }

    #[test]
    fn prompt_text_is_one_literal_argument() {
        let text = "Application \"personae-vault\" asks:\nquotes ' and $() `literal` café";
        for backend in ["kdialog", "zenity"] {
            let c = command(
                backend,
                "title ' \"",
                text,
                MessageBoxType::YesNo,
                IconType::Question,
                0,
            )
            .unwrap();
            assert_eq!(c.get_program(), backend);
            assert_eq!(c.get_args().filter(|a| *a == text).count(), 1);
            assert!(!c.get_args().any(|a| a == "-c"));
        }
    }

    #[test]
    fn successful_explicit_yes_and_no_are_distinct_from_failures() {
        assert_eq!(
            answer(MessageBoxType::YesNo, Some(0), b"yes\n"),
            Ok(MessageBoxButton::Ok)
        );
        assert_eq!(
            answer(MessageBoxType::YesNo, Some(0), b"no\n"),
            Ok(MessageBoxButton::Cancel)
        );
        for code in [1, 2] {
            assert_eq!(
                answer(MessageBoxType::YesNo, Some(code), b"no\n"),
                Err(MessageBoxError::NoDecision)
            );
        }
        for code in [Some(127), Some(255), None] {
            assert_eq!(
                answer(MessageBoxType::YesNo, code, b""),
                Err(MessageBoxError::BackendFailed(code))
            );
        }
    }

    #[test]
    fn successful_but_invalid_output_cannot_answer() {
        for output in [b"".as_slice(), b"help text", b"yes\nno", b" yes", &[255]] {
            assert_eq!(
                answer(MessageBoxType::YesNo, Some(0), output),
                Err(MessageBoxError::InvalidResponse)
            );
        }
    }

    #[test]
    fn cancel_and_third_button_defaults_keep_their_meaning() {
        assert_eq!(selected(MessageBoxType::OkCancel, 0), "cancel");
        assert_eq!(selected(MessageBoxType::YesNoCancel, 0), "cancel");
        assert_eq!(selected(MessageBoxType::YesNoCancel, 2), "no");
        assert_eq!(selected(MessageBoxType::YesNo, 1), "yes");
        assert_eq!(
            answer(MessageBoxType::YesNoCancel, Some(0), b"no"),
            Ok(MessageBoxButton::No)
        );
        assert_eq!(
            answer(MessageBoxType::YesNoCancel, Some(0), b"cancel"),
            Ok(MessageBoxButton::Cancel)
        );
    }

    #[test]
    fn unavailable_backend_and_unexpected_notice_output_refuse() {
        assert!(matches!(
            command(
                "basicinput",
                "title",
                "message",
                MessageBoxType::YesNo,
                IconType::Question,
                0
            ),
            Err(MessageBoxError::Unavailable)
        ));
        assert_eq!(
            answer(MessageBoxType::Ok, Some(0), b""),
            Ok(MessageBoxButton::Ok)
        );
        assert_eq!(
            answer(MessageBoxType::Ok, Some(0), b"no"),
            Err(MessageBoxError::InvalidResponse)
        );
    }
}
