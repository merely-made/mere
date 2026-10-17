
#![allow(dead_code,unused_variables,unused_mut,unused_assignments)]
use std::env;
const VERSION:&str="audit dispatch fixture";
mod genet_host_api {
#[derive(Clone,Copy,Debug)]pub enum EngineProfile{Livery,Scripted}
impl std::str::FromStr for EngineProfile{type Err=String;fn from_str(s:&str)->Result<Self,String>{match s.to_ascii_lowercase().as_str(){"livery"=>Ok(Self::Livery),"scripted"=>Ok(Self::Scripted),_=>Err("unknown fixture engine".into())}}}
pub struct DeferredShellEngine;impl DeferredShellEngine{pub fn new(_:EngineProfile)->Self{Self}}
pub struct Capabilities{pub javascript:bool,pub webdriver:bool,pub devtools:bool,pub webgpu:bool,pub webxr:bool}
pub trait ShellEngine{fn capabilities(&self)->Capabilities;}
impl ShellEngine for DeferredShellEngine{fn capabilities(&self)->Capabilities{Capabilities{javascript:false,webdriver:false,devtools:false,webgpu:false,webxr:false}}}
}
use genet_host_api::{DeferredShellEngine,EngineProfile,ShellEngine};
mod inker{pub mod routing{pub const ENGINE_GENET_READER:&str="genet.reader";pub const ENGINE_GENET_SCRIPTED:&str="genet.scripted";pub const ENGINE_GENET_SCRIPTED_NOVA:&str="genet.scripted.nova";}}
mod pelt_desktop {
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ProductReceipt {
    /// Ordinary script-free article with linked CSS, font, images, tables, and
    /// an in-page jump-link interaction.
    Article,
    /// A nested overflow region beside retained editable form controls.
    Controls,
    /// Viewport-driven grid reflow beside a retained two-column table.
    Responsive,
    /// Parser mutation, a scheduled timer turn, and controller-owned document
    /// navigation through the scripted Livery/Buckram route.
    Scripted,
    /// Initial Text Fragment activation in a retained Livery document.
    TextFragment,
    /// Redirected document with a linked/imported stylesheet resource graph.
    Resources,
    /// Held Gemtext lowered by Nematic and navigated through the retained
    /// smolweb session controller.
    Gemtext,
}

impl ProductReceipt {
    pub fn id(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Controls => "controls",
            Self::Responsive => "responsive",
            Self::Scripted => "scripted",
            Self::TextFragment => "text-fragment",
            Self::Resources => "resources",
            Self::Gemtext => "gemtext",
        }
    }

    pub fn needs_scripted_profile(self) -> bool {
        matches!(self, Self::Scripted)
    }

    pub fn default_size(self) -> (u32, u32) {
        match self {
            Self::Article
            | Self::Controls
            | Self::Responsive
            | Self::Scripted
            | Self::TextFragment
            | Self::Resources
            | Self::Gemtext => (960, 640),
        }
    }

    pub fn default_frames(self) -> u32 {
        match self {
            Self::Article
            | Self::Controls
            | Self::Responsive
            | Self::Scripted
            | Self::TextFragment
            | Self::Resources
            | Self::Gemtext => 3,
        }
    }
}


#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum WorkspaceReceipt {
    /// A Gemtext link replaces only its own tile through capability routing.
    Mixed,
    /// An unavailable external engine visibly falls back to the owned Livery lane.
    Fallback,
    /// P6 host chrome controls a focused tile while the P5 mixed workspace remains live.
    Chrome,
    /// P6's document-lane loading and failed-navigation projection, without a
    /// native surface dependency.
    LoadingError,
    /// P6's Pelt-owned appearance drawer. This named receipt uses in-memory
    /// storage, while callers may supply a durable Pelt store.
    Appearance,
    /// P6's retained Chrome/Frisket AccessKit tree and typed action routing.
    Accessibility,
    /// P6's address TextInput SetValue route through the installed platform
    /// bridge, including retained loading and error documents.
    AccessibilityAddress,
    /// P7's one-tree Livery child semantics, namespaced and routed through the
    /// same Pelt input path as its content aperture.
    AccessibilityChildren,
    /// P7's native Livery text-control SetValue route, including retained
    /// value projection and ordinary form submission.
    AccessibilityEdit,
    /// P7's nested-scroll Livery ScrollIntoView route through Pelt's
    /// composite accessibility namespace.
    AccessibilityScroll,
    /// P7's clip-aware nested Livery Click route after an explicit reveal.
    AccessibilityClick,
    /// P7's nested Livery editor input and physical text-selection route.
    AccessibilityInput,
    /// P6's compact Chrome layout at a small logical viewport.
    NarrowChrome,
    /// P6's actual high-DPI Chrome input and capture alignment.
    ChromeDpi,
    /// Reader extracts a focused tile's already-held source response in the
    /// shared workspace, then releases it back to its original Livery route.
    Reader,
    /// Reader's already-presented partial link tree is namespaced under each
    /// Pelt content aperture. Focus is virtual until a later host source
    /// handoff can supply destination bodies for Reader navigation.
    ReaderAccessibility,
    /// A developer-only Tabard palette preview over the retained Pelt Chrome.
    /// This does not persist a Pelt appearance preference or recolor documents.
    TabardPreview,
    /// A developer-only Tabard palette preview over Reader's existing
    /// host-theme seam. This does not alter Fleece or persist a Pelt theme.
    TabardReaderPreview,
}

impl WorkspaceReceipt {
    pub fn id(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::Fallback => "fallback",
            Self::Chrome => "chrome",
            Self::LoadingError => "loading-error",
            Self::Appearance => "appearance",
            Self::Accessibility => "accessibility",
            Self::AccessibilityAddress => "accessibility-address",
            Self::AccessibilityChildren => "accessibility-children",
            Self::AccessibilityEdit => "accessibility-edit",
            Self::AccessibilityScroll => "accessibility-scroll",
            Self::AccessibilityClick => "accessibility-click",
            Self::AccessibilityInput => "accessibility-input",
            Self::NarrowChrome => "narrow-chrome",
            Self::ChromeDpi => "chrome-dpi",
            Self::Reader => "reader",
            Self::ReaderAccessibility => "reader-accessibility",
            Self::TabardPreview => "tabard-preview",
            Self::TabardReaderPreview => "tabard-reader-preview",
        }
    }

}
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectedEngine {
    Livery,
    Scripted,
    Reader,
}

impl SelectedEngine {
    fn profile(self) -> EngineProfile {
        match self {
            Self::Livery | Self::Reader => EngineProfile::Livery,
            Self::Scripted => EngineProfile::Scripted,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Livery => "livery",
            Self::Scripted => "scripted",
            Self::Reader => "reader",
        }
    }
}

fn source_prefix() {
    let mut selected_engine = SelectedEngine::Livery;
    let mut url = None;
    // JS backend for `--engine scripted` (boa default; nova needs --features
    // scripted-nova). Parsed as a string so the flag exists even in builds without
    // the scripted profile.
    let mut js_engine = String::from("boa");
    // Physical client size for headed viewers.
    let mut size: Option<(u32, u32)> = None;
    // Bounded headed capture/smoke run. Interactive profiles leave this unset.
    let mut frames: Option<u32> = None;
    let mut product_receipt: Option<pelt_desktop::ProductReceipt> = None;
    let mut artifact: Option<std::path::PathBuf> = None;
    let mut workspace_size_matrix: Option<Vec<(u32, u32)>> = None;
    let mut with_tiles = false;
    let mut tile_receipt = false;
    let mut capability_receipt = false;
    let mut tearout_receipt = false;
    let mut tearout_cancellation_receipt = false;
    let mut workspace_receipt: Option<pelt_desktop::WorkspaceReceipt> = None;
    let mut appearance_store: Option<std::path::PathBuf> = None;
    let mut theme_library: Option<std::path::PathBuf> = None;
    let mut tile_engine_overrides = Vec::new();
    let mut tile_urls = Vec::new();
    let mut netrender_smoke = false;
    let mut webgl_wgpu_smoke = false;
    #[cfg(feature = "windows-present")]
    let mut windows_present_smoke = false;
    #[cfg(feature = "windows-present")]
    let mut windows_present_surfaces_smoke = false;
    #[cfg(feature = "macos-present")]
    let mut macos_present_smoke = false;
    #[cfg(feature = "macos-present")]
    let mut macos_present_surfaces_smoke = false;
    #[cfg(feature = "linux-present")]
    let mut wayland_present_smoke = false;
    #[cfg(feature = "linux-present")]
    let mut wayland_present_surfaces_smoke = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            },
            "--version" => {
                println!("{VERSION}");
                return;
            },
            "--engine" => {
                let Some(value) = args.next() else {
                    eprintln!("--engine requires livery, reader, or scripted");
                    std::process::exit(2);
                };
                selected_engine = parse_engine(&value);
            },
            value if value.starts_with("--engine=") => {
                selected_engine = parse_engine(&value["--engine=".len()..]);
            },
            "--js" => {
                let Some(value) = args.next() else {
                    eprintln!("--js requires boa or nova");
                    std::process::exit(2);
                };
                js_engine = value;
            },
            value if value.starts_with("--js=") => {
                js_engine = value["--js=".len()..].to_owned();
            },
            "--size" => {
                let Some(value) = args.next() else {
                    eprintln!("--size requires WxH in physical pixels");
                    std::process::exit(2);
                };
                size = Some(parse_size(&value));
            },
            value if value.starts_with("--size=") => {
                size = Some(parse_size(&value["--size=".len()..]));
            },
            "--frames" => {
                let Some(value) = args.next() else {
                    eprintln!("--frames requires a positive integer");
                    std::process::exit(2);
                };
                frames = Some(parse_frames(&value));
            },
            value if value.starts_with("--frames=") => {
                frames = Some(parse_frames(&value["--frames=".len()..]));
            },
            "--product-receipt" => {
                let Some(value) = args.next() else {
                    eprintln!("--product-receipt requires a named receipt id");
                    std::process::exit(2);
                };
                product_receipt = Some(parse_product_receipt(&value));
            },
            value if value.starts_with("--product-receipt=") => {
                product_receipt = Some(parse_product_receipt(&value["--product-receipt=".len()..]));
            },
            "--artifact" => {
                let Some(value) = args.next() else {
                    eprintln!("--artifact requires a PNG path");
                    std::process::exit(2);
                };
                artifact = Some(value.into());
            },
            value if value.starts_with("--artifact=") => {
                artifact = Some(value["--artifact=".len()..].into());
            },
            "--appearance-store" => {
                let Some(value) = args.next() else {
                    eprintln!("--appearance-store requires a caller-selected file path");
                    std::process::exit(2);
                };
                appearance_store = Some(value.into());
                with_tiles = true;
            },
            value if value.starts_with("--appearance-store=") => {
                appearance_store = Some(value["--appearance-store=".len()..].into());
                with_tiles = true;
            },
            "--theme-library" => {
                let Some(value) = args.next() else {
                    eprintln!("--theme-library requires a file path");
                    std::process::exit(2);
                };
                theme_library = Some(value.into());
                with_tiles = true;
            },
            value if value.starts_with("--theme-library=") => {
                theme_library = Some(value["--theme-library=".len()..].into());
                with_tiles = true;
            },
            "--tiles" | "--workspace" => {
                with_tiles = true;
            },
            "--tile-receipt" => {
                with_tiles = true;
                tile_receipt = true;
            },
            "--capability-receipt" => {
                with_tiles = true;
                capability_receipt = true;
            },
            "--workspace-tearout-receipt" => {
                with_tiles = true;
                tearout_receipt = true;
            },
            "--workspace-tearout-cancellation-receipt" => {
                with_tiles = true;
                tearout_cancellation_receipt = true;
            },
            "--workspace-receipt" => {
                let Some(value) = args.next() else {
                    eprintln!(
                        "--workspace-receipt requires mixed, fallback, chrome, loading-error, appearance, accessibility, accessibility-address, accessibility-children, accessibility-edit, accessibility-scroll, accessibility-click, accessibility-input, narrow-chrome, chrome-dpi, reader, reader-accessibility, tabard-preview, or tabard-reader-preview"
                    );
                    std::process::exit(2);
                };
                workspace_receipt = Some(parse_workspace_receipt(&value));
                with_tiles = true;
            },
            value if value.starts_with("--workspace-receipt=") => {
                workspace_receipt = Some(parse_workspace_receipt(
                    &value["--workspace-receipt=".len()..],
                ));
                with_tiles = true;
            },
            "--workspace-size-matrix" => {
                let Some(value) = args.next() else {
                    eprintln!("--workspace-size-matrix requires comma-separated WxH values");
                    std::process::exit(2);
                };
                workspace_size_matrix = Some(parse_workspace_size_matrix(&value));
            },
            value if value.starts_with("--workspace-size-matrix=") => {
                workspace_size_matrix = Some(parse_workspace_size_matrix(
                    &value["--workspace-size-matrix=".len()..],
                ));
            },
            "--tile-engine" => {
                let Some(value) = args.next() else {
                    eprintln!("--tile-engine requires N=engine-id");
                    std::process::exit(2);
                };
                tile_engine_overrides.push(parse_tile_engine(&value));
                with_tiles = true;
            },
            value if value.starts_with("--tile-engine=") => {
                tile_engine_overrides.push(parse_tile_engine(&value["--tile-engine=".len()..]));
                with_tiles = true;
            },
            "--netrender-smoke" => {
                netrender_smoke = true;
            },
            "--webgl-wgpu-smoke" => {
                webgl_wgpu_smoke = true;
            },
            #[cfg(feature = "windows-present")]
            "--windows-present-smoke" => {
                windows_present_smoke = true;
            },
            #[cfg(feature = "windows-present")]
            "--windows-present-surfaces-smoke" => {
                windows_present_surfaces_smoke = true;
            },
            #[cfg(feature = "macos-present")]
            "--macos-present-smoke" => {
                macos_present_smoke = true;
            },
            #[cfg(feature = "macos-present")]
            "--macos-present-surfaces-smoke" => {
                macos_present_surfaces_smoke = true;
            },
            #[cfg(feature = "linux-present")]
            "--wayland-present-smoke" => {
                wayland_present_smoke = true;
            },
            #[cfg(feature = "linux-present")]
            "--wayland-present-surfaces-smoke" => {
                wayland_present_surfaces_smoke = true;
            },
            value if value.starts_with('-') => {
                eprintln!("unsupported pelt option: {value}");
                std::process::exit(2);
            },
            value => {
                url = Some(value.to_owned());
                tile_urls.push(value.to_owned());
            },
        }
    }

    if let Some(receipt) = product_receipt {
        let receipt_engine = if receipt.needs_scripted_profile() {
            SelectedEngine::Scripted
        } else {
            SelectedEngine::Livery
        };
        if selected_engine != receipt_engine {
            eprintln!(
                "product receipt {} is owned by the {} profile",
                receipt.id(),
                receipt_engine.label()
            );
            std::process::exit(2);
        }
        if with_tiles {
            eprintln!(
                "product receipt {} is a single-document receipt",
                receipt.id()
            );
            std::process::exit(2);
        }
        if url.is_some() {
            eprintln!("product receipt {} owns its fixture URL", receipt.id());
            std::process::exit(2);
        }
        if artifact.is_none() {
            eprintln!("--product-receipt {} needs --artifact <png>", receipt.id());
            std::process::exit(2);
        }
        if receipt == pelt_desktop::ProductReceipt::Gemtext && !cfg!(feature = "smolweb") {
            eprintln!("--product-receipt gemtext needs --features smolweb");
            std::process::exit(2);
        }
        url = Some(product_receipt_fixture(receipt));
    } else if artifact.is_some() && workspace_receipt.is_none() {
        eprintln!("--artifact is only accepted with a named receipt");
        std::process::exit(2);
    }

    if let Err(error) = validate_workspace_size_matrix(
        workspace_receipt,
        workspace_size_matrix.as_deref(),
        size.is_some(),
    ) {
        eprintln!("{error}");
        std::process::exit(2);
    }

    let engine_profile = selected_engine.profile();
    let engine = DeferredShellEngine::new(engine_profile);
    let capabilities = engine.capabilities();
    let url = if with_tiles {
        tile_urls
            .first()
            .cloned()
            .unwrap_or_else(|| "about:blank".to_owned())
    } else {
        url.unwrap_or_else(|| "about:blank".to_owned())
    };
    println!(
        "pelt host profile={} url={} javascript={} webdriver={} devtools={} webgpu={} webxr={}",
        selected_engine.label(),
        url,
        capabilities.javascript,
        capabilities.webdriver,
        capabilities.devtools,
        capabilities.webgpu,
        capabilities.webxr
    );

    if netrender_smoke {
        // Presentation-backend smoke: it is independent of the selected
        // document engine and exits after its bounded receipt.
        run_optional_netrender_smoke();
        return;
    }

    if webgl_wgpu_smoke {
        run_optional_webgl_wgpu_smoke();
        return;
    }

    #[cfg(feature = "windows-present")]
    if windows_present_smoke {
        run_optional_windows_present_smoke();
        return;
    }

    #[cfg(feature = "windows-present")]
    if windows_present_surfaces_smoke {
        run_optional_windows_present_surfaces_smoke();
        return;
    }

    #[cfg(feature = "macos-present")]
    if macos_present_smoke {
        run_optional_macos_present_smoke();
        return;
    }

    #[cfg(feature = "macos-present")]
    if macos_present_surfaces_smoke {
        run_optional_macos_present_surfaces_smoke();
        return;
    }

    #[cfg(feature = "linux-present")]
    if wayland_present_smoke {
        run_optional_wayland_present_smoke();
        return;
    }

    #[cfg(feature = "linux-present")]
    if wayland_present_surfaces_smoke {
        run_optional_wayland_present_surfaces_smoke();
        return;
    }


    println!("AUDIT owner dispatch reached");
}
fn parse_size(value: &str) -> (u32, u32) {
    match parse_size_value(value) {
        Ok(size) => size,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        },
    }
}

fn parse_size_value(value: &str) -> Result<(u32, u32), String> {
    let Some((width, height)) = value.split_once(['x', 'X']) else {
        return Err(format!(
            "--size expects WxH in physical pixels (got '{value}')"
        ));
    };
    let width = width.parse::<u32>().ok().filter(|value| *value > 0);
    let height = height.parse::<u32>().ok().filter(|value| *value > 0);
    match (width, height) {
        (Some(width), Some(height)) => Ok((width, height)),
        _ => Err(format!(
            "--size expects positive WxH dimensions (got '{value}')"
        )),
    }
}

fn parse_workspace_size_matrix(value: &str) -> Vec<(u32, u32)> {
    match parse_workspace_size_matrix_value(value) {
        Ok(sizes) => sizes,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        },
    }
}

fn parse_workspace_size_matrix_value(value: &str) -> Result<Vec<(u32, u32)>, String> {
    if value.trim().is_empty() {
        return Err("--workspace-size-matrix requires at least two sizes".to_owned());
    }
    let mut sizes = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        if item.is_empty() {
            return Err("--workspace-size-matrix expects comma-separated WxH values".to_owned());
        }
        let size = parse_size_value(item)
            .map_err(|error| error.replacen("--size", "--workspace-size-matrix", 1))?;
        if sizes.contains(&size) {
            return Err(format!(
                "--workspace-size-matrix contains duplicate size {}x{}",
                size.0, size.1
            ));
        }
        sizes.push(size);
    }
    if sizes.len() < 2 {
        return Err("--workspace-size-matrix requires at least two sizes".to_owned());
    }
    Ok(sizes)
}

fn validate_workspace_size_matrix(
    receipt: Option<pelt_desktop::WorkspaceReceipt>,
    sizes: Option<&[(u32, u32)]>,
    has_single_size: bool,
) -> Result<(), String> {
    let Some(_sizes) = sizes else {
        return Ok(());
    };
    if receipt != Some(pelt_desktop::WorkspaceReceipt::Mixed) {
        return Err("--workspace-size-matrix requires --workspace-receipt mixed".to_owned());
    }
    if has_single_size {
        return Err(
            "--workspace-size-matrix cannot be combined with --size; put the initial size first"
                .to_owned(),
        );
    }
    Ok(())
}

/// Parse a deterministic headed-frame limit. Zero would open and immediately close a
/// window without proving it presented, so it is deliberately rejected.
fn parse_frames(value: &str) -> u32 {
    match value.parse::<u32>().ok().filter(|value| *value > 0) {
        Some(value) => value,
        None => {
            eprintln!("--frames expects a positive integer (got '{value}')");
            std::process::exit(2);
        },
    }
}

fn parse_product_receipt(value: &str) -> pelt_desktop::ProductReceipt {
    match value {
        "article" => pelt_desktop::ProductReceipt::Article,
        "controls" => pelt_desktop::ProductReceipt::Controls,
        "responsive" => pelt_desktop::ProductReceipt::Responsive,
        "scripted" => pelt_desktop::ProductReceipt::Scripted,
        "text-fragment" => pelt_desktop::ProductReceipt::TextFragment,
        "resources" => pelt_desktop::ProductReceipt::Resources,
        "gemtext" => pelt_desktop::ProductReceipt::Gemtext,
        _ => {
            eprintln!(
                "--product-receipt expects article, controls, responsive, scripted, text-fragment, resources, or gemtext (got '{value}')"
            );
            std::process::exit(2);
        },
    }
}

fn product_receipt_fixture(receipt: pelt_desktop::ProductReceipt) -> String {
    match receipt {
        pelt_desktop::ProductReceipt::Article => std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("livery-route")
            .join("index.html")
            .to_string_lossy()
            .into_owned(),
        pelt_desktop::ProductReceipt::Controls => std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("p5-controls")
            .join("index.html")
            .to_string_lossy()
            .into_owned(),
        pelt_desktop::ProductReceipt::Responsive => {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join("p5-responsive")
                .join("index.html")
                .to_string_lossy()
                .into_owned()
        },
        pelt_desktop::ProductReceipt::Scripted => std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("p5-scripted")
            .join("index.html")
            .to_string_lossy()
            .into_owned(),
        pelt_desktop::ProductReceipt::Resources => std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("p5-resources")
            .join("start")
            .join("index.html")
            .to_string_lossy()
            .into_owned(),
        pelt_desktop::ProductReceipt::TextFragment => {
            let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join("text-fragment")
                .join("index.html")
                .to_string_lossy()
                .into_owned();
            format!("{fixture}#:~:text=The%20retained%20text%20fragment%20target")
        },
        pelt_desktop::ProductReceipt::Gemtext => {
            "gemini://pelt.test/p5-gemtext/index.gmi".to_owned()
        },
    }
}

fn parse_tile_engine(value: &str) -> (u64, String) {
    let Some((tile, engine)) = value.split_once('=') else {
        eprintln!("--tile-engine expects N=engine-id (got '{value}')");
        std::process::exit(2);
    };
    let tile = tile.parse::<u64>().ok().filter(|tile| *tile > 0);
    if let (Some(tile), false) = (tile, engine.trim().is_empty()) {
        (tile, engine.trim().to_owned())
    } else {
        eprintln!("--tile-engine expects N=engine-id (got '{value}')");
        std::process::exit(2);
    }
}

fn parse_workspace_receipt(value: &str) -> pelt_desktop::WorkspaceReceipt {
    match value {
        "mixed" => pelt_desktop::WorkspaceReceipt::Mixed,
        "fallback" => pelt_desktop::WorkspaceReceipt::Fallback,
        "chrome" => pelt_desktop::WorkspaceReceipt::Chrome,
        "loading-error" => pelt_desktop::WorkspaceReceipt::LoadingError,
        "appearance" => pelt_desktop::WorkspaceReceipt::Appearance,
        "accessibility" => pelt_desktop::WorkspaceReceipt::Accessibility,
        "accessibility-address" => pelt_desktop::WorkspaceReceipt::AccessibilityAddress,
        "accessibility-children" => pelt_desktop::WorkspaceReceipt::AccessibilityChildren,
        "accessibility-edit" => pelt_desktop::WorkspaceReceipt::AccessibilityEdit,
        "accessibility-scroll" => pelt_desktop::WorkspaceReceipt::AccessibilityScroll,
        "accessibility-click" => pelt_desktop::WorkspaceReceipt::AccessibilityClick,
        "accessibility-input" => pelt_desktop::WorkspaceReceipt::AccessibilityInput,
        "narrow-chrome" => pelt_desktop::WorkspaceReceipt::NarrowChrome,
        "chrome-dpi" => pelt_desktop::WorkspaceReceipt::ChromeDpi,
        "reader" => pelt_desktop::WorkspaceReceipt::Reader,
        "reader-accessibility" => pelt_desktop::WorkspaceReceipt::ReaderAccessibility,
        "tabard-preview" => pelt_desktop::WorkspaceReceipt::TabardPreview,
        "tabard-reader-preview" => pelt_desktop::WorkspaceReceipt::TabardReaderPreview,
        _ => {
            eprintln!(
                "--workspace-receipt expects mixed, fallback, chrome, loading-error, appearance, accessibility, accessibility-address, accessibility-children, accessibility-edit, accessibility-scroll, accessibility-click, accessibility-input, narrow-chrome, chrome-dpi, reader, reader-accessibility, tabard-preview, or tabard-reader-preview (got '{value}')"
            );
            std::process::exit(2);
        },
    }
}

fn parse_engine(value: &str) -> SelectedEngine {
    if value.eq_ignore_ascii_case("reader") || value.eq_ignore_ascii_case("genet.reader") {
        return SelectedEngine::Reader;
    }
    match value.parse() {
        Ok(EngineProfile::Livery) => SelectedEngine::Livery,
        Ok(EngineProfile::Scripted) => SelectedEngine::Scripted,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        },
    }
}

fn print_help(){println!("AUDIT help");}
fn run_optional_netrender_smoke(){println!("AUDIT netrender called");}
fn run_optional_webgl_wgpu_smoke(){println!("AUDIT webgl called");}
fn workspace_override(js_engine:&str)->Option<&'static str>{let selected_engine=SelectedEngine::Scripted;
            let profile_engine = match selected_engine {
                SelectedEngine::Livery => None,
                SelectedEngine::Reader => {
                    if !cfg!(feature = "reader") {
                        eprintln!("the reader workspace route needs `--features reader`");
                        std::process::exit(2);
                    }
                    Some(inker::routing::ENGINE_GENET_READER)
                },
                SelectedEngine::Scripted => {
                    if !cfg!(feature = "scripted") {
                        eprintln!("the scripted workspace route needs `--features scripted`");
                        std::process::exit(2);
                    }
                    if js_engine == "nova" {
                        if !cfg!(feature = "scripted-nova") {
                            eprintln!("the Nova workspace route needs `--features scripted-nova`");
                            std::process::exit(2);
                        }
                        Some(inker::routing::ENGINE_GENET_SCRIPTED_NOVA)
                    } else {
                        Some(inker::routing::ENGINE_GENET_SCRIPTED)
                    }
                },
            };

profile_engine}
fn main(){if let Ok(js)=env::var("AUDIT_WORKSPACE_JS"){println!("AUDIT workspace engine={:?}",workspace_override(&js));return;}source_prefix();}
