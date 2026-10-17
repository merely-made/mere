// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Workspace Pool over the existing origin store, independent of graph sessions.
use super::*;
use cambium::{TextInput, button, lens, span, text_field_typed};
use inker::pool::Pool;
use muniment::{Backend, IndexedDbBackend, WriteOp};
const SLOT: &str = "workspace/pool/v1";
type Completion = Result<(Pool, Vec<u8>), String>;
pub(super) struct Pane {
    data: Pool,
    backend: Option<IndexedDbBackend>,
    original: Option<Vec<u8>>,
    blocked: Option<String>,
    completed: Rc<RefCell<Option<Completion>>>,
    busy: bool,
    pub(super) wake: Option<cambium_rootstock::HostWake>,
    open: bool,
    draft: TextInput,
    status: String,
    export_url: Option<String>,
}
impl Pane {
    pub(super) async fn open(product: Option<&product::SavedProduct>) -> Self {
        let backend = product.and_then(|p| p.forme_source()).map(|(b, _)| b);
        let mut pane = Self {
            data: Pool::default(),
            backend,
            original: None,
            blocked: None,
            completed: Default::default(),
            busy: false,
            wake: None,
            open: false,
            draft: TextInput::default(),
            status: String::new(),
            export_url: None,
        };
        if let Some(backend) = &pane.backend {
            let loaded = async {
                let bytes = backend.get(SLOT).await.map_err(|e| e.to_string())?;
                if let Some(bytes) = bytes {
                    let pool: Pool = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    pool.validate()?;
                    Ok::<_, String>(Some((bytes, pool)))
                } else {
                    Ok(None)
                }
            }
            .await;
            match loaded {
                Ok(Some((bytes, pool))) => {
                    pane.original = Some(bytes);
                    pane.data = pool;
                },
                Ok(None) => {},
                Err(e) => {
                    pane.blocked = Some(format!(
                        "Pool could not be opened: {e}. Stored contents are unchanged."
                    ))
                },
            }
        }
        pane
    }
    fn commit(&mut self, candidate: Pool) -> Result<(), String> {
        if self.busy {
            return Err("Pool is saving; wait before changing another item".into());
        }
        if let Some(error) = &self.blocked {
            return Err(error.clone());
        }
        candidate.validate()?;
        let backend = self.backend.clone().ok_or("Pool storage is unavailable")?;
        let expected = self.original.clone();
        let bytes = serde_json::to_vec(&candidate).map_err(|e| e.to_string())?;
        let completed = self.completed.clone();
        let wake = self.wake.clone();
        self.busy = true;
        self.status = "Saving Pool…".into();
        wasm_bindgen_futures::spawn_local(async move {
            let result=async {
                let accepted=Rc::new(Cell::new(false)); let flag=accepted.clone(); let saved=bytes.clone();
                backend.transact(Box::new(move |reader| {
                    match reader.get(SLOT) {
                        Ok(current) if current==expected=> {flag.set(true);vec![WriteOp::Put{key:SLOT.into(),value:saved}]},
                        _=>vec![],
                    }
                })).await.map_err(|e|e.to_string())?;
                if !accepted.get() {return Err("The stored Pool changed; reopen this workspace before changing its contents".into());}
                Ok((candidate,bytes))
            }.await;
            *completed.borrow_mut() = Some(result);
            if let Some(wake) = wake {
                wake.wake();
            }
        });
        Ok(())
    }
    pub(super) fn ready(&self) -> bool {
        self.completed.borrow().is_some()
    }
    pub(super) fn poll(&mut self) {
        let Some(result) = self.completed.borrow_mut().take() else {
            return;
        };
        self.busy = false;
        match result {
            Ok((data, bytes)) => {
                self.data = data;
                self.original = Some(bytes);
                self.status = "Pool saved in this workspace".into();
            },
            Err(e) => self.status = e,
        }
    }
}
fn add_scene(page: &mut TreePage) {
    let result = (|| {
        let pane = page.backdrop.borrow();
        let doc = pane
            .document
            .as_ref()
            .ok_or("Open a portable scene first")?;
        let class = match &doc.primary.facts {
            scenomise::backdrops::SceneFacts::MetreGrid(_) => "metre",
            scenomise::backdrops::SceneFacts::TemplatedSequence(_) => "progression",
            _ => "scene",
        };
        let text = doc.to_djot(class)?;
        let title = doc.primary.title.clone();
        let source = Some(doc.primary.source.id.clone());
        drop(pane);
        let mut candidate = page.pool.data.clone();
        candidate.add(title, "text/djot".into(), text, source)?;
        page.pool.commit(candidate)
    })();
    if let Err(e) = result {
        page.pool.status = e;
    }
}
fn add_text(page: &mut TreePage) {
    let result = (|| {
        let text = page.pool.draft.text().to_string();
        let title = text
            .lines()
            .next()
            .unwrap_or("Material")
            .chars()
            .take(60)
            .collect();
        let mut candidate = page.pool.data.clone();
        candidate.add(title, "text/plain".into(), text, None)?;
        page.pool.commit(candidate)
    })();
    if let Err(e) = result {
        page.pool.status = e;
    }
}
fn load_scene(page: &mut TreePage, id: uuid::Uuid) {
    let result = (|| {
        let text = page.pool.data.get(id)?.text.clone();
        let mut pane = page.backdrop.borrow_mut();
        if pane.original.is_some() {
            return Err(
                "Finish or discard the current scene copy before opening Pool material".into(),
            );
        }
        pane.install(&text)?;
        pane.sample = None;
        pane.open = true;
        Ok(())
    })();
    page.pool.status = match result {
        Ok(()) => "Opened independent Pool material · held content is unchanged".into(),
        Err(e) => e,
    };
}
impl Drop for Pane {
    fn drop(&mut self) {
        if let Some(url) = self.export_url.take() {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    }
}
fn export(page: &mut TreePage, id: uuid::Uuid) {
    let result = (|| {
        let item = page.pool.data.get(id)?;
        let extension = if item.media_type == "text/djot" {
            "djot"
        } else {
            "txt"
        };
        let url = backdrop::export_material(
            item.text.as_bytes(),
            &item.media_type,
            &format!("pool-{id}.{extension}"),
        )?;
        if let Some(old) = page.pool.export_url.replace(url) {
            let _ = web_sys::Url::revoke_object_url(&old);
        }
        Ok::<_, String>(())
    })();
    page.pool.status = match result {
        Ok(()) => "Exported independent Pool material".into(),
        Err(e) => e,
    };
}
pub(super) fn view(page: &TreePage) -> Option<Child> {
    if page.product.is_none() {
        return None;
    }
    let mut rows: Vec<Child> = vec![Box::new(
        el(
            "div",
            (
                button("Pool", |p: &mut TreePage, _| p.pool.open = !p.pool.open)
                    .attr("aria-expanded", page.pool.open.to_string()),
                span(format!(
                    "{} {}",
                    page.pool.data.items.len(),
                    if page.pool.data.items.len() == 1 {
                        "item"
                    } else {
                        "items"
                    }
                )),
            ),
        )
        .attr("style", "display:flex;gap:8px;align-items:center"),
    )];
    if page.pool.open {
        rows.push(Box::new(span(
            "Workspace material · independent copies · kept until removed",
        )));
        rows.push(Box::new(el("div", (
            button("Add scene to Pool", |p: &mut TreePage, _| add_scene(p)),
            lens(
                |field: &mut TextInput| {
                    text_field_typed(field)
                        .attr("aria-label", "Pool text")
                        .attr("placeholder", "Text to keep in Pool")
                        .attr("id", "pool-text")
                        .attr("style", "min-width:0;max-width:100%;width:22rem;border:1px solid;padding:6px")
                },
                |p: &mut TreePage| &mut p.pool.draft,
            ),
            button("Add text to Pool", |p: &mut TreePage, _| add_text(p)),
        )).attr("style", "display:flex;flex-wrap:wrap;align-items:center;gap:8px")));
        if page.pool.data.items.is_empty() && page.pool.blocked.is_none() {
            rows.push(Box::new(span("Pool is empty.")));
        }
        for item in &page.pool.data.items {
            let id = item.id;
            let scene = scenomise::document::SceneDocument::parse_text(&item.text).is_ok();
            let mut card: Vec<Child> = vec![Box::new(el("strong", item.title.clone()))];
            if !scene {
                card.push(Box::new(span(
                    item.text.chars().take(180).collect::<String>(),
                )));
            }
            let mut actions: Vec<Child> = Vec::new();
            if scene {
                actions.push(Box::new(
                    button("Open Pool scene", move |p: &mut TreePage, _| {
                        load_scene(p, id)
                    })
                    .attr("data-pool-id", id.to_string()),
                ));
            }
            actions.push(Box::new(button(
                "Export Pool item",
                move |p: &mut TreePage, _| export(p, id),
            )));
            actions.push(Box::new(
                button("Remove from Pool", move |p: &mut TreePage, _| {
                    let result = (|| {
                        let mut candidate = p.pool.data.clone();
                        candidate.remove(id)?;
                        p.pool.commit(candidate)
                    })();
                    if let Err(e) = result {
                        p.pool.status = e;
                    }
                })
                .attr("data-pool-id", id.to_string()),
            ));
            card.push(Box::new(
                el("div", actions).attr("style", "display:flex;flex-wrap:wrap;gap:12px"),
            ));
            if let Some(source) = &item.source {
                card.push(Box::new(span(format!("Source: {source}"))));
            }
            rows.push(Box::new(el("article",card).attr("style","display:flex;flex-direction:column;gap:6px;padding:8px;border:1px solid;border-radius:6px;overflow-wrap:anywhere")));
        }
        if let Some(e) = &page.pool.blocked {
            rows.push(Box::new(span(e.clone()).attr("role", "status")));
        }
        rows.push(Box::new(
            span(page.pool.status.clone()).attr("role", "status"),
        ));
    }
    Some(Box::new(el("section",rows).attr("aria-label","Pool").attr("style","display:flex;flex-direction:column;gap:8px;flex:none;max-height:35vh;overflow:auto;padding:8px")))
}
