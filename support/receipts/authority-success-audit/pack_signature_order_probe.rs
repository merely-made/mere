//! Native public pack verification, synthetic signing key and manifest only.
use eidetic::{ManifestId, SignatureRef, TrustEnvelope};
use eidetic::pack::{PackManifest, PackPart, PackPartRole, PackVerdict, sign_pack, verify_pack};
use personae::Ed25519Keypair;
fn fixture() -> (PackManifest, SignatureRef) {
    let key=Ed25519Keypair::from_seed([0x29;32]);
    let author=key.public_key().to_bytes().iter().map(|b|format!("{b:02x}")).collect();
    let pack=PackManifest {name:"synthetic audit pack".into(),version:"1".into(),author,
        requested_scopes:vec![],parts:vec![PackPart{name:"asset.txt".into(),role:PackPartRole::Asset,
        blob:ManifestId::of_blob(b"synthetic asset"),bytes:15}]};
    let signature=sign_pack(&pack,&key);(pack,signature)
}
fn envelope(signatures:Vec<SignatureRef>)->TrustEnvelope {
    let mut value=TrustEnvelope::self_asserted();value.signatures=signatures;value
}

#[test]
fn permuting_one_valid_and_one_malformed_binding_changes_the_verdict() {
    let (pack,valid)=fixture();let bad=SignatureRef("personae:ed25519:malformed".into());
    let valid_first=verify_pack(&pack,&envelope(vec![valid.clone(),bad.clone()]));
    let invalid_first=verify_pack(&pack,&envelope(vec![bad,valid]));
    assert_eq!(valid_first,PackVerdict::Trusted);
    assert_eq!(invalid_first,PackVerdict::Broken);
    println!("OBSERVATION: the same signature multiset is Trusted with the valid binding first and Broken with the malformed binding first");
}

#[test]
fn valid_invalid_and_unsigned_single_binding_controls_hold() {
    let (pack,valid)=fixture();
    assert_eq!(verify_pack(&pack,&envelope(vec![valid])),PackVerdict::Trusted);
    assert_eq!(verify_pack(&pack,&envelope(vec![SignatureRef("personae:ed25519:malformed".into())])),PackVerdict::Broken);
    assert_eq!(verify_pack(&pack,&envelope(vec![])),PackVerdict::Unsigned);
    println!("CONTROL: valid, malformed and unsigned single-binding classifications remain distinct");
}

#[test]
fn signature_still_refuses_a_changed_manifest() {
    let (mut pack,valid)=fixture();pack.version="changed".into();
    assert_eq!(verify_pack(&pack,&envelope(vec![valid])),PackVerdict::Broken);
    println!("CONTROL: a signature over the original manifest does not authorize a changed version");
}
