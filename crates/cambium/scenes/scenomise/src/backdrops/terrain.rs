// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn compile_terrain(
    out: &mut BackdropScene,
    src: sceno::SourceIx,
    map: &TerrainCoverage,
) {
    out.scene.bounds = Rect::new(Vec2::ZERO, map.extent);
    for points in &map.contours {
        guide(out, src, points.clone());
    }
    let mut ids = std::collections::HashMap::new();
    for d in &map.devices {
        if let Some(at) = d.position {
            let id = mark(
                out,
                d.key.clone(),
                &d.source,
                d.label.clone(),
                format!("{}; placed in source coordinates", d.realm),
                MarkRole::Node,
                at,
                Footprint::Circle { radius: 5.0 },
                true,
                Some(Vec2::new(at.x, at.y + 24.0)),
            );
            out.scene.items.last_mut().unwrap().hit = Some(Footprint::Circle { radius: 22.0 });
            ids.insert(&d.key, id);
        } else {
            out.unplaced.push(Unplaced {
                key: d.key.clone(),
                source: d.source.clone(),
                label: d.label.clone(),
                reason: "Position unknown; no coverage can be anchored".into(),
            });
        }
    }
    for c in &map.coverage {
        let region = out.scene.regions.len();
        out.scene.regions.push(Region {
            members: vec![ids[&c.device]],
            space: Scene::WORLD,
            contour: Some(Footprint::Polygon {
                points: c.contour.clone(),
            }),
            label: Some(format!("coverage:{:?}", c.tier)),
            confidence: 1.0,
        });
        out.fields.push(FieldReading {
            region,
            device: Some(c.device.clone()),
            tier: Some(c.tier),
            source: c.source.clone(),
            basis: c.basis.clone(),
        });
    }
    for points in &map.unmeasured {
        let region = out.scene.regions.len();
        out.scene.regions.push(Region {
            members: vec![],
            space: Scene::WORLD,
            contour: Some(Footprint::Polygon {
                points: points.clone(),
            }),
            label: Some("unmeasured".into()),
            confidence: 1.0,
        });
        out.fields.push(FieldReading {
            region,
            device: None,
            tier: None,
            source: out.scene.sources[src.0 as usize].clone(),
            basis: "Source declares this region unmeasured".into(),
        });
    }
    for link in &map.links {
        let (Some(&from), Some(&to)) = (ids.get(&link.from), ids.get(&link.to)) else {
            out.diagnostics.push(format!(
                "{} → {}: route has an unplaced endpoint",
                link.from, link.to
            ));
            continue;
        };
        let a = out.scene.items[from.0 as usize].transform.translate;
        let b = out.scene.items[to.0 as usize].transform.translate;
        relation(
            out,
            from,
            to,
            vec![a, b],
            match link.kind {
                LinkKind::Authored => "authored",
                LinkKind::Derived => "derived",
            },
            format!("{} → {}", link.from, link.to),
            link.why.clone(),
        );
    }
}
