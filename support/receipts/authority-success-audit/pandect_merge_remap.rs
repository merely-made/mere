//! Diagnostic: merge identity remapping rewrites an ordinary literal title.
use pandect::{Author, CapturedDelta, MereSessions};
use uuid::Uuid;
use std::collections::BTreeMap;

#[test]
fn merging_a_pick_rewrites_title_text_that_happens_to_equal_a_node_id() {
    pollster::block_on(async {
        let sessions = MereSessions::new(muniment::MemoryBackend::new());
        let author = Author::person("audit");
        let manifest = sessions.mint(author.clone(), None).await.unwrap();
        let mut target = sessions.open(manifest.session_id).await.unwrap();
        let original_id = Uuid::from_u128(301);
        target.apply(author.clone(), vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: original_id.to_string(), url: "https://audit.test/item".into(), position: [0.0, 0.0],
        }]).await.unwrap();
        let picked = sessions.fork_component(&target, original_id, author.clone()).await.unwrap();
        let mut source = sessions.open(picked.session_id).await.unwrap();
        let copied_id = source.graph().nodes().next().unwrap().1.id;
        assert_ne!(original_id, copied_id);
        let requested_title = copied_id.to_string();
        source.apply(author.clone(), vec![CapturedDelta::ReplaySetNodeTitleById {
            node_id: copied_id.to_string(), title: requested_title.clone(),
        }]).await.unwrap();
        let plan = sessions.merge_in(&target, &source).await.unwrap();
        assert!(plan.is_clean());
        sessions.apply_merge(&mut target, &mut source, plan, &BTreeMap::new(), author).await.unwrap();
        let actual = &target.graph().get_node_by_id(original_id).unwrap().1.title;
        assert_eq!(actual, &original_id.to_string(), "identity remapping also rewrote ordinary title data");
        assert_ne!(actual, &requested_title);
        assert_eq!(source.graph().get_node_by_id(copied_id).unwrap().1.title, requested_title, "source literal remains exactly what the user wrote");
    });
}
