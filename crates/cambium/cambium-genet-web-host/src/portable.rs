// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Compose complete portable sources in painter order without raster intermediates.
use cambium_rootstock::PortablePaintCapture;
use netrender::vello_backends::{
    SparseFragmentLimits, SparseFragmentRegistry, SparseResourceLimits, VelloBackend,
};
use netrender::{NO_CLIP, Scene, SceneClip, SceneFragment, SceneLayer, SceneOp, Transform};
use paint_list_render::bundle::{BundleError, translate_capture};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct PortablePresentationError {
    pub source_key: Option<u64>,
    pub message: String,
}
impl std::fmt::Display for PortablePresentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "portable source {:?}: {}", self.source_key, self.message)
    }
}
impl std::error::Error for PortablePresentationError {}
fn failure(key: Option<u64>, message: impl std::fmt::Display) -> PortablePresentationError {
    PortablePresentationError {
        source_key: key,
        message: message.to_string(),
    }
}
fn bundle_failure(error: BundleError) -> PortablePresentationError {
    let key = match error {
        BundleError::MissingProducer(key)
        | BundleError::DuplicateProducer(key)
        | BundleError::ForeignProducer(key)
        | BundleError::StaleProducer(key)
        | BundleError::InvalidProducer { key, .. } => Some(key),
        _ => None,
    };
    failure(key, error)
}
fn affine(sx: f32, sy: f32, tx: f32, ty: f32) -> Transform {
    Transform {
        m: [
            sx, 0., 0., 0., 0., sy, 0., 0., 0., 0., 1., 0., tx, ty, 0., 1.,
        ],
    }
}
fn clip(rect: [f32; 4], radii: [f32; 4], transform_id: u32) -> SceneOp {
    let mut layer = SceneLayer::clip(SceneClip::Rect { rect, radii });
    layer.transform_id = transform_id;
    SceneOp::PushLayer(layer)
}

pub fn materialize(
    capture: &PortablePaintCapture,
    width: u32,
    height: u32,
    scale: f32,
    limits: SparseResourceLimits,
) -> Result<Scene, PortablePresentationError> {
    if !scale.is_finite() || scale <= 0. || width == 0 || height == 0 {
        return Err(failure(None, "invalid physical viewport or scale"));
    }
    let translated = translate_capture(capture).map_err(bundle_failure)?;
    if !translated.box_shadow_masks.is_empty() {
        return Err(failure(
            None,
            "generated shadow masks need an explicit portable source",
        ));
    }
    let external: BTreeMap<_, _> = translated
        .external_textures
        .iter()
        .map(|draw| (draw.image_key, draw.texture_key))
        .collect();
    let mut reserved: BTreeSet<u64> = translated.scene.image_sources.keys().copied().collect();
    reserved.extend(external.keys().copied());
    let mut next_key = u64::MAX;
    let mut registry = SparseFragmentRegistry::new(SparseFragmentLimits::default());
    let mut sources = BTreeMap::new();
    let mut ordered: Vec<_> = capture.producers.iter().collect();
    ordered.sort_unstable_by_key(|source| source.texture_key);
    for source in ordered {
        let key = source.texture_key;
        let mut scene = source.scene.clone();
        let intrinsic = source
            .physical_size
            .map(|size| size as f32 / source.layout_scale);
        let alpha = scene.root_alpha;
        let blend = scene.root_blend_mode;
        scene.root_alpha = 1.;
        scene.root_blend_mode = netrender::SceneBlendMode::Normal;
        let mut root = SceneLayer::alpha(alpha);
        root.blend_mode = blend;
        scene.ops.insert(0, SceneOp::PushLayer(root));
        scene
            .ops
            .insert(1, clip([0., 0., intrinsic[0], intrinsic[1]], [0.; 4], 0));
        scene.ops.push(SceneOp::PopLayer);
        scene.ops.push(SceneOp::PopLayer);
        // Producer image identities are local. Allocate stable, disjoint keys
        // in source/key order; never overwrite a parent's or sibling's pixels.
        let mut images = BTreeMap::new();
        for old in scene.image_sources.keys().copied().collect::<BTreeSet<_>>() {
            while reserved.contains(&next_key) {
                next_key = next_key
                    .checked_sub(1)
                    .ok_or_else(|| failure(Some(key), "image key space exhausted"))?;
            }
            images.insert(old, next_key);
            reserved.insert(next_key);
        }
        scene.image_sources = scene
            .image_sources
            .into_iter()
            .map(|(old, data)| (images[&old], data))
            .collect();
        for op in &mut scene.ops {
            match op {
                SceneOp::Image(image) => image.key = images[&image.key],
                SceneOp::Pattern(pattern) => pattern.tile = images[&pattern.tile],
                _ => {},
            }
        }
        let id = registry
            .register_fragment(SceneFragment::from_scene(scene))
            .map_err(|error| failure(Some(key), error))?;
        let mut check = Scene::new(source.scene.viewport_width, source.scene.viewport_height);
        check.place_fragment(id, Transform::IDENTITY);
        registry
            .resolve(&check, VelloBackend::WebGl2, limits)
            .map_err(|error| failure(Some(key), error))?;
        sources.insert(key, (id, intrinsic));
    }
    let mut parent = translated.scene.clone();
    parent.ops.clear();
    for op in translated.scene.ops {
        let SceneOp::Image(image) = &op else {
            parent.ops.push(op);
            continue;
        };
        let Some(key) = external.get(&image.key).copied() else {
            parent.ops.push(op);
            continue;
        };
        let (id, intrinsic) = sources
            .get(&key)
            .copied()
            .ok_or_else(|| failure(Some(key), "missing producer"))?;
        let opacity = image.color[3];
        if image.color[..3] != [opacity; 3] {
            return Err(failure(Some(key), "external producer tint is not neutral"));
        }
        let [u0, v0, u1, v1] = image.uv;
        if u1 <= u0 || v1 <= v0 {
            return Err(failure(Some(key), "invalid producer UV extent"));
        }
        let sx = (image.x1 - image.x0) / (intrinsic[0] * (u1 - u0));
        let sy = (image.y1 - image.y0) / (intrinsic[1] * (v1 - v0));
        let local = affine(
            sx,
            sy,
            image.x0 - u0 * intrinsic[0] * sx,
            image.y0 - v0 * intrinsic[1] * sy,
        );
        let placement = local.then(&parent.transforms[image.transform_id as usize]);
        let primitive_clip = image.clip_rect != NO_CLIP;
        if primitive_clip {
            parent
                .ops
                .push(clip(image.clip_rect, image.clip_corner_radii, 0));
        }
        parent.ops.push(clip(
            [image.x0, image.y0, image.x1, image.y1],
            [0.; 4],
            image.transform_id,
        ));
        parent
            .ops
            .push(SceneOp::PushLayer(SceneLayer::alpha(opacity)));
        parent.place_fragment(id, placement);
        parent.ops.push(SceneOp::PopLayer);
        parent.ops.push(SceneOp::PopLayer);
        if primitive_clip {
            parent.ops.push(SceneOp::PopLayer);
        }
    }
    let mut flattened = registry
        .resolve(&parent, VelloBackend::WebGl2, limits)
        .map_err(|error| failure(None, error))?;
    // Applying one final placement also transforms device-space primitive
    // clips through the existing resolver. Index zero remains identity.
    let alpha = flattened.root_alpha;
    let blend = flattened.root_blend_mode;
    flattened.root_alpha = 1.;
    flattened.root_blend_mode = netrender::SceneBlendMode::Normal;
    let mut physical = Scene::new(width, height);
    physical.root_alpha = alpha;
    physical.root_blend_mode = blend;
    let mut scaled = SparseFragmentRegistry::new(SparseFragmentLimits::default());
    let id = scaled
        .register_fragment(SceneFragment::from_scene(flattened))
        .map_err(|error| failure(None, error))?;
    physical.place_fragment(id, affine(scale, scale, 0., 0.));
    scaled
        .resolve(&physical, VelloBackend::WebGl2, limits)
        .map_err(|error| failure(None, error))
}
