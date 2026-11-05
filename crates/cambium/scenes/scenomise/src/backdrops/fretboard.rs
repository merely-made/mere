// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn compile_fretboard(out: &mut BackdropScene, src: sceno::SourceIx, board: &Fretboard) {
    let cell = 84.0;
    let top = 48.0;
    let open = 62.0;
    let nut = 94.0;
    let width = nut + board.frets as f32 * cell + 24.0;
    let height = top + board.strings.len() as f32 * 48.0;
    out.scene.bounds = Rect::new(Vec2::ZERO, Size2::new(width, height));
    for (s, string) in board.strings.iter().enumerate() {
        let y = top + s as f32 * 48.0;
        guide(
            out,
            src,
            vec![Vec2::new(44.0, y), Vec2::new(width - 16.0, y)],
        );
        out.guides.push(GuideLabel {
            at: Vec2::new(24.0, y),
            text: string.label.clone(),
        });
    }
    for fr in 0..=board.frets {
        let x = nut + fr as f32 * cell;
        guide(
            out,
            src,
            vec![Vec2::new(x, top - 18.0), Vec2::new(x, height - 24.0)],
        );
        if fr < board.frets {
            out.guides.push(GuideLabel {
                at: Vec2::new(x + cell / 2.0, 20.0),
                text: (fr + 1).to_string(),
            });
        }
    }
    out.guides.push(GuideLabel {
        at: Vec2::new(open, 20.0),
        text: "open".into(),
    });
    let mut ids = Vec::new();
    for p in &board.positions {
        let x = if p.fret == 0 {
            open
        } else {
            nut + (p.fret as f32 - 0.5) * cell
        };
        let y = top + p.string_index as f32 * 48.0;
        ids.push(mark(
            out,
            p.key.clone(),
            &p.source,
            p.label.clone(),
            format!(
                "string {}, fret {}, pitch class {}",
                p.string_index + 1,
                p.fret,
                p.pitch_class
            ),
            match p.role {
                ToneRole::Root => MarkRole::Root,
                ToneRole::Third => MarkRole::Third,
                ToneRole::Fifth => MarkRole::Fifth,
                ToneRole::Other => MarkRole::Other,
            },
            Vec2::new(x, y),
            Footprint::Circle { radius: 5.0 },
            true,
            Some(Vec2::new(x, y + 24.0)),
        ));
        out.scene.items.last_mut().unwrap().hit = Some(Footprint::Rect {
            size: Size2::new(40.0, 40.0),
        });
    }
    for b in 0..board.positions.len() {
        if let Some(a) = (0..b).find(|&a| board.positions[a].source == board.positions[b].source) {
            let pa = out.scene.items[ids[a].0 as usize].transform.translate;
            let pb = out.scene.items[ids[b].0 as usize].transform.translate;
            relation(out,ids[a],ids[b],vec![pa,pb],"same-resource",board.positions[b].label.clone(),"One source resource realized at different instrument positions; octaves may differ".into());
        }
    }
}
