pub use cambium::{CompositionEvent,Key,KeyEvent,NamedKey};
mod text_input {
#[path = "C:/Users/mark_/Code/repos/mere/crates/cambium/cambium/src/controls/text_input/command.rs"] mod command;
#[path = "C:/Users/mark_/Code/repos/mere/crates/cambium/cambium/src/controls/text_input/core.rs"] mod core;
#[path = "C:/Users/mark_/Code/repos/mere/crates/cambium/cambium/src/controls/text_input/multiline.rs"] mod multiline;
#[path = "C:/Users/mark_/Code/repos/mere/crates/cambium/cambium/src/controls/text_input/word_motion.rs"] mod word_motion;
pub use command::{CaretMove,TextCommand,TextFieldMode,text_command_from_key};
pub(crate) use core::TextSnapshot;
pub use core::{CaretAffinity,CaretPosition,CaretSelection,Composition,TextInput};
}
mod controls {
    pub use super::text_input::TextInput;
    pub(crate) use super::text_input::TextSnapshot;
}
#[path = "../../../crates/cambium/cambium/src/editor.rs"]
mod editor;
use text_input::*;
fn main() {
    let mut t=TextInput::new("a");t.apply(TextCommand::Insert("\u{301}".into()));
    assert_eq!(t.text(),"a\u{301}");assert_eq!(t.caret(),2);
    assert_eq!(genet_text::segments(t.text(),genet_text::BoundaryKind::ExtendedGrapheme).count(),1);
    t.apply(TextCommand::Backspace);assert_eq!(t.text(),"a\u{301}");
    println!("PASS separately inserted combining mark leaves caret beyond grapheme count and first Backspace deletes no text");
    t.apply(TextCommand::Backspace);assert_eq!(t.text(),"");
    println!("PASS second Backspace finally removes combined grapheme");
    let mut t=TextInput::new("a\u{301}");assert_eq!(t.caret(),1);
    t.apply(TextCommand::Backspace);assert_eq!(t.text(),"");
    println!("PASS precombined buffer control deletes the same grapheme in one Backspace");
    let mut t=TextInput::new("ab");t.set_caret_byte(1,false);
    t.apply(TextCommand::Insert("\u{301}".into()));assert_eq!(t.caret_position().byte,t.text().len());
    t.apply(TextCommand::Insert("x".into()));assert_eq!(t.text(),"a\u{301}bx");
    println!("PASS combining insertion before suffix jumps over b and next character appends after b");
    let mut t=TextInput::new("ab");t.set_caret_byte(1,false);
    t.apply(TextCommand::Insert("x".into()));assert_eq!(t.text(),"axb");assert_eq!(t.caret_position().byte,2);
    println!("PASS ordinary insertion control stays before suffix");
    let mut t=TextInput::new("a");t.set_caret_byte(0,false);t.apply(TextCommand::SetComposition{text:"候".into(),selection:None});
    assert_eq!(t.render_text(),"候a");
    assert!(!t.apply(TextCommand::Move{movement:CaretMove::GraphemeLeft,extend:false}));
    assert!(t.composition().is_none());assert_eq!(t.render_text(),"a");
    println!("PASS no-op caret move clears composition and changes render while reporting false");
    t.apply(TextCommand::SetComposition{text:"候".into(),selection:None});
    assert!(t.apply(TextCommand::CancelComposition));
    println!("PASS explicit composition cancellation control reports true");
}
