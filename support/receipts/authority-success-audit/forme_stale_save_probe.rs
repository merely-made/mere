use graphshell::{
    forme_session::{FormeSession, SaveKind},
    forme_workspace::FormeWorkspace,
    presentation_editor::{PresentationSession, SaveKind as RuleSave, presentation::PrimitiveKind},
};
use mere::kernel::graph::Graph;
use muniment::MemoryBackend;
use uuid::Uuid;

#[test]
fn independent_stale_forme_save_overwrites_an_acknowledged_layout() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let owner = Uuid::from_u128(0xf041);
        let graph = Graph::new();
        FormeWorkspace::new(owner).save(store.clone()).await.unwrap();
        let mut first = FormeSession::load(store.clone(), owner, &graph).await.unwrap();
        let mut stale = FormeSession::load(store.clone(), owner, &graph).await.unwrap();
        first.unlock().unwrap();
        stale.unlock().unwrap();
        first.edit(None, 1, |w| { w.bounds[0] -= 10.; Ok(()) }).unwrap();
        stale.edit(None, 1, |w| { w.bounds[1] -= 20.; Ok(()) }).unwrap();
        let a = first.prepare_save(SaveKind::Apply, 2).unwrap();
        let b = stale.prepare_save(SaveKind::Apply, 2).unwrap();
        let result = a.save(store.clone()).await;
        first.finish_save(a, result).unwrap();
        assert_eq!(first.committed().bounds[0], -250.);
        let result = b.save(store.clone()).await;
        stale.finish_save(b, result).unwrap();
        let reopened = FormeSession::load(store, owner, &graph).await.unwrap();
        assert_eq!(reopened.committed().bounds, [-240., -170., 240., 150.]);
        assert_ne!(reopened.committed().bounds[0], first.committed().bounds[0]);
        assert_eq!(first.committed().document.id, stale.committed().document.id);
    });
}

#[test]
fn rule_workspace_refuses_the_equivalent_stale_baseline_and_keeps_its_draft() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let owner = Uuid::from_u128(0xf042);
        let mut first = PresentationSession::load(store.clone(), owner).await.unwrap();
        let mut stale = PresentationSession::load(store.clone(), owner).await.unwrap();
        for (s, label) in [(&mut first, "first"), (&mut stale, "second")] {
            s.begin().unwrap();
            s.edit(None, 1, |d| { let i = d.add(PrimitiveKind::Node); d.rules[i].label = label.into(); }).unwrap();
        }
        let a = first.prepare_save(RuleSave::Apply).unwrap();
        let b = stale.prepare_save(RuleSave::Apply).unwrap();
        first.finish_save(a.save(store.clone()).await).unwrap();
        let refused = b.save(store.clone()).await;
        assert!(refused.as_ref().unwrap_err().contains("changed elsewhere"));
        assert!(stale.finish_save(refused).is_err());
        assert!(stale.dirty());
        let reopened = PresentationSession::load(store, owner).await.unwrap();
        assert_eq!(reopened.committed().rules[0].label, "first");
    });
}
