//! Public counterexamples over a freshly compiled unchanged complete Servitor.
use chartulary::{Container, EditSpec, GraphLog, Relation};
use identity::{IdentityProvider, InMemoryProvider};
use insigne::delegation::SignedDelegationCertificate;
use servitor_audit::{AuthorityProvider, Cap, DelegationTable, Gate, Grant, GrantTable, Mode, ScopePath,
    Subject, WatchEvent, WatchTable};

fn signed_namespace_is_not_consulted() {
    let root = InMemoryProvider::from_seed([91; 32]);
    let holder = InMemoryProvider::from_seed([92; 32]);
    let key = root.master_public_key().to_bytes();
    let subject = Subject::new(holder.master_public_key().to_bytes());
    let needed = Cap::scope("trail/today").unwrap();
    let make = |domain: &str, resource: &[u8]| {
        let mut certificate = servitor_audit::delegation::root_certificate(
            key, subject, &Cap::scope("trail").unwrap(), Mode::Write, resource.to_vec(),
            10, None, 0, [1; 32]);
        certificate.scope.domain = domain.to_owned();
        let signed = SignedDelegationCertificate::issue(&root, certificate).unwrap();
        signed.check().unwrap();
        let mut table = DelegationTable::new(key);
        table.set_now(20);
        table.adopt(signed);
        table
    };
    let normal = make(servitor_audit::delegation::DENIZEN_DOMAIN, b"world-A");
    assert!(normal.covers(subject, &needed, Mode::Write));
    assert!(!normal.covers(subject, &Cap::scope("vault").unwrap(), Mode::Write));
    let foreign = make("unrelated.application", b"world-Z");
    assert!(foreign.covers(subject, &needed, Mode::Write));
    assert_eq!(foreign.certificates().next().unwrap().certificate.scope.domain, "unrelated.application");
    assert_eq!(foreign.certificates().next().unwrap().certificate.scope.resource, b"world-Z");
    let mut fresh_world = GraphLog::<Container, Relation>::new();
    Gate::new().petition(&foreign, &mut fresh_world, subject, &ScopePath::parse("trail").unwrap(), 0,
        vec![EditSpec::InsertNode(Container::new("trail/accepted"))]).unwrap();
    assert_eq!(fresh_world.graph().node_count(), 1);
    println!("PASS valid signed foreign domain/resource covers and commits; normal scope and outside-scope controls checked");
}

fn truncated_subject_labels_collide() {
    let a = Subject::new([0xab; 32]);
    let mut b_bytes = [0xcd; 32];
    b_bytes[..4].copy_from_slice(&a.0[..4]);
    let b = Subject::new(b_bytes);
    assert_ne!(a, b);
    assert_eq!(a.to_author(), b.to_author());
    let auth = GrantTable::new()
        .with_grant(Grant::new(a, Cap::scope("trail").unwrap(), Mode::Write))
        .with_grant(Grant::new(b, Cap::scope("trail").unwrap(), Mode::Write));
    let gate = Gate::new();
    let mut nested = GraphLog::<Container, Relation>::new();
    gate.petition(&auth, &mut nested, a, &ScopePath::parse("trail").unwrap(), 0,
        vec![EditSpec::InsertNode(Container::new("trail/a"))]).unwrap();
    let rev = nested.revision();
    gate.petition(&auth, &mut nested, b, &ScopePath::parse("trail").unwrap(), rev,
        vec![EditSpec::InsertNode(Container::new("trail/b"))]).unwrap();
    assert_eq!(nested.log().entries()[0].author, nested.log().entries()[1].author);
    let mut watches = WatchTable::new();
    let label_a = format!("denizen:{}", &a.to_hex()[..8]);
    let label_b = format!("denizen:{}", &b.to_hex()[..8]);
    watches.register(&auth, a, ScopePath::parse("trail").unwrap(), label_a).unwrap();
    let scopes = [ScopePath::parse("trail/b").unwrap()];
    assert!(watches.wake(&[WatchEvent { seq: 1, author: &label_b, scopes: &scopes }]).is_empty());
    assert_eq!(watches.wake(&[WatchEvent { seq: 2, author: "different-author", scopes: &scopes }]).len(), 1);
    println!("PASS distinct raw Subject values collapse to one commit author and suppress another subject's wake; different-label control wakes");
}

fn watch_survives_expired_authority() {
    let subject = Subject::new([3; 32]);
    let need = Cap::scope("private").unwrap();
    let mut auth = GrantTable::new().with_grant(Grant::new(subject, need.clone(), Mode::Read).expiring_at(10));
    auth.set_now(9);
    let mut watches = WatchTable::new();
    watches.register(&auth, subject, ScopePath::parse("private").unwrap(), "self").unwrap();
    auth.set_now(10);
    assert!(!auth.covers(subject, &need, Mode::Read));
    let scopes = [ScopePath::parse("private/change").unwrap()];
    let event = WatchEvent { seq: 1, author: "someone-else", scopes: &scopes };
    assert_eq!(watches.would_wake(&[event]), vec![subject]);
    assert_eq!(watches.wake(&[event])[0].matched, vec![1]);
    watches.remove_subject(subject);
    assert!(watches.wake(&[WatchEvent { seq: 2, ..event }]).is_empty());
    println!("PASS expired read grant still wakes and reveals matched seq; explicit removal control stops it");
}

fn overlapping_watches_duplicate_a_subject() {
    let subject = Subject::new([4; 32]);
    let auth = GrantTable::new().with_grant(Grant::new(subject, Cap::scope("trail").unwrap(), Mode::Read));
    let mut watches = WatchTable::new();
    for scope in ["trail", "trail/today"] {
        watches.register(&auth, subject, ScopePath::parse(scope).unwrap(), "self").unwrap();
    }
    let scopes = [ScopePath::parse("trail/today/a").unwrap()];
    let event = WatchEvent { seq: 1, author: "someone-else", scopes: &scopes };
    assert_eq!(watches.would_wake(&[event]), vec![subject]);
    let wakes = watches.wake(&[event]);
    assert_eq!(wakes.len(), 2);
    assert!(wakes.iter().all(|wake| wake.subject == subject && wake.matched == [1]));
    assert!(watches.wake(&[event]).is_empty());
    println!("PASS peek names one subject but drain returns it twice for overlapping watches; replay cursor control is quiet");
}

fn main() {
    signed_namespace_is_not_consulted();
    truncated_subject_labels_collide();
    watch_survives_expired_authority();
    overlapping_watches_duplicate_a_subject();
}
