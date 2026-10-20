// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Optional Djot transport for the existing portable scene-document format.
//! Domain declarations remain separate from the JSON payload's facts. This is
//! an inert codec, without a domain resolver, evaluation, I/O or source writes.
use super::{SCENE_DOCUMENT_FORMAT, SceneDocument};
use crate::backdrops::MAX_SCENE_INPUT_BYTES;
use jotdown::{Container, Event, Parser};

impl SceneDocument {
    pub fn to_djot(&self, class: &str) -> Result<String, String> {
        self.validate()?;
        if class.is_empty()
            || !class
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
        {
            return Err("Scene block needs a simple domain class".into());
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let source = format!(
            "{{format=\"{SCENE_DOCUMENT_FORMAT}\"}}\n::: {class}\n```json\n{json}\n```\n:::\n"
        );
        if source.len() > MAX_SCENE_INPUT_BYTES {
            return Err("Scene block exceeds one MiB".into());
        }
        Ok(source)
    }

    /// Read one declared scene block. Unrelated text/blocks are not evaluated;
    /// multiple scene blocks require an explicit selection by the caller.
    pub fn from_djot(source: &str) -> Result<Self, String> {
        if source.len() > MAX_SCENE_INPUT_BYTES {
            return Err("Scene block exceeds one MiB".into());
        }
        let mut depth = 0usize;
        let mut target = None;
        let mut count = 0;
        let mut payloads = Vec::new();
        let mut code = None::<String>;
        let mut pending_format = false;
        for event in Parser::new(source) {
            match event {
                Event::Attributes(attrs) => {
                    pending_format = attrs
                        .get_value("format")
                        .is_some_and(|s| s.to_string() == SCENE_DOCUMENT_FORMAT);
                },
                Event::Start(Container::Div { .. }, attrs) => {
                    depth += 1;
                    if attrs
                        .get_value("format")
                        .is_some_and(|s| s.to_string() == SCENE_DOCUMENT_FORMAT)
                        || pending_format
                    {
                        count += 1;
                        target = Some(depth);
                    }
                    pending_format = false;
                },
                Event::End(Container::Div { .. }) => {
                    if target == Some(depth) {
                        target = None;
                    }
                    depth = depth.saturating_sub(1);
                },
                Event::Start(Container::CodeBlock { language }, _)
                    if target == Some(depth) && language == "json" =>
                {
                    code = Some(String::new())
                },
                Event::Str(text) if code.is_some() => code.as_mut().unwrap().push_str(&text),
                Event::End(Container::CodeBlock { .. }) => {
                    if let Some(code) = code.take() {
                        payloads.push(code);
                    }
                },
                _ => pending_format = false,
            }
        }
        if count != 1 || payloads.len() != 1 {
            return Err("Choose one declared scene block with one JSON body".into());
        }
        Self::parse(&payloads[0])
    }

    /// The same facts may arrive as standalone JSON or an attributed Djot block.
    pub fn parse_text(source: &str) -> Result<Self, String> {
        Self::parse(source).or_else(|_| Self::from_djot(source))
    }
}
