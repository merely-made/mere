// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Actual shared surface → Netrender → Vello CPU frames; no browser/GPU claim.
use netrender::vello_backends::{CpuSession, SparseResourceLimits};
use pictograph::canvas::backdrop_surface::{BackdropSurface, SurfaceTheme};
use scenomise::{backdrops::SceneInput, document::SceneDocument};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("supply an output directory")?,
    );
    std::fs::create_dir_all(&output)?;
    let fixtures = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../support/design-studies/backdrops")
        });
    let font =
        include_bytes!("../../../cambium/examples/genet_web_smoke/assets/Roboto-Regular.ttf");
    let mut receipt = Vec::new();
    let mut samples = vec![
        ("metre", "line-1"),
        ("fretboard", "string-5-fret-3"),
        ("terrain", "workshop"),
    ];
    for name in ["progression", "progression-g"] {
        if fixtures.join(format!("{name}.json")).is_file() {
            samples.push((name, "bar-0"));
        }
    }
    let mut inputs = Vec::new();
    for (name, selected) in samples {
        let input = SceneInput::parse(&std::fs::read_to_string(
            fixtures.join(format!("{name}.json")),
        )?)?;
        inputs.push((name.to_owned(), selected.to_owned(), input));
    }
    for name in ["arrangement-regular", "arrangement-varied"] {
        let path = fixtures.join(format!("{name}.json"));
        if !path.is_file() {
            continue;
        }
        let document = SceneDocument::parse(&std::fs::read_to_string(path)?)?;
        inputs.push((name.to_owned(), "card-3".into(), document.primary.clone()));
        for view in document.linked {
            inputs.push((
                format!("{name}-{}", view.occurrence),
                String::new(),
                view.scene,
            ));
        }
    }
    for (name, selected, input) in inputs {
        let mut surface = BackdropSurface::new(&input)?;
        let original_reading = serde_json::to_value(surface.scene())?;
        surface.register_font(font.to_vec());
        let selected = if surface
            .scene()
            .marks
            .iter()
            .any(|m| m.selectable && m.key == selected)
        {
            selected.to_owned()
        } else {
            surface
                .scene()
                .marks
                .iter()
                .find(|m| m.selectable)
                .ok_or("recipe has no placed selectable occurrence")?
                .key
                .clone()
        };
        surface.select(Some(&selected))?;
        let scene = surface.paint(900, 540, &SurfaceTheme::default())?;
        let mut cpu = CpuSession::new(SparseResourceLimits::default());
        let pixmap = cpu.render(&scene)?;
        let pixels = pixmap.data_as_u8_slice();
        assert_eq!(pixels.len(), 900 * 540 * 4);
        assert!(
            pixels.chunks_exact(4).any(|p| p != &pixels[0..4]),
            "frame is not blank"
        );
        std::fs::write(output.join(format!("{name}.rgba")), pixels)?;
        std::fs::write(
            output.join(format!("{name}.scene.json")),
            serde_json::to_vec_pretty(surface.scene())?,
        )?;
        std::fs::write(
            output.join(format!("{name}.recipe.json")),
            serde_json::to_vec_pretty(&input)?,
        )?;
        let first = pixels.to_vec();
        let next = surface
            .scene()
            .marks
            .iter()
            .find(|m| m.selectable && m.key != selected)
            .map(|m| m.key.clone());
        surface.select(next.as_deref())?;
        let next_scene = surface.paint(900, 540, &SurfaceTheme::default())?;
        let next_pixels = cpu.render(&next_scene)?;
        assert_ne!(
            first,
            next_pixels.data_as_u8_slice(),
            "selection changes the real raster"
        );
        let source_before = serde_json::to_value(surface.scene())?;
        surface.dismiss_retained();
        assert_eq!(source_before, serde_json::to_value(surface.scene())?);
        if name == "terrain" && surface.scene().unplaced.iter().any(|m| m.key == "pocket") {
            let sample =
                |pixels: &[u8]| pixels[(444 * 900 + 730) * 4..(444 * 900 + 730) * 4 + 4].to_vec();
            assert_eq!(
                sample(&first),
                sample(next_pixels.data_as_u8_slice()),
                "unmeasured ground masks all coverage choices"
            );
            surface.select(Some("pocket"))?;
            let unknown = surface.paint(900, 540, &SurfaceTheme::default())?;
            let unknown = cpu.render(&unknown)?;
            assert_eq!(sample(&first), sample(unknown.data_as_u8_slice()));
        }
        receipt.push(serde_json::json!({"scene":name,"width":900,"height":540,"generation":surface.scene().scene.generation,"items":surface.scene().scene.items.len(),"sources":surface.scene().scene.sources.len(),"backdrops":surface.scene().scene.backdrops.len(),"selection_changes_pixels":true,"source_unchanged_by_attention":true}));
        assert_eq!(original_reading, serde_json::to_value(surface.scene())?);
    }
    std::fs::write(
        output.join("receipt.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"backend":"Vello CPU","presentation":"not exercised","frames":receipt}),
        )?,
    )?;
    println!(
        "Production surface frames and selection controls passed: {}",
        output.display()
    );
    Ok(())
}
