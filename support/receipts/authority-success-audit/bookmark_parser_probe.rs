use eidetic::import::{parse_bookmark_items, ImportedBookmarkItem};

fn folder_input(name: Option<&str>) -> String {
    let name=name.map(|s| format!("\"name\":{},",serde_json::to_string(s).unwrap())).unwrap_or_default();
    format!(r#"{{"roots":{{"bookmark_bar":{{"children":[{{"type":"folder","name":"Outer","children":[{{"type":"folder",{name}"children":[{{"type":"url","url":"https://inner.test/"}}]}},{{"type":"url","url":"https://sibling.test/"}}]}}]}}}}}}"#)
}
fn path(item: &ImportedBookmarkItem) -> Vec<&str> { item.folder_path.iter().map(|p|p.label.as_str()).collect() }

#[test]
fn whitespace_named_folder_pops_its_parent_before_the_following_sibling() {
    let items=parse_bookmark_items(&folder_input(Some("   "))).unwrap();
    assert_eq!(items.len(),2);
    assert_eq!(path(&items[0]),vec!["Outer"]);
    assert!(path(&items[1]).is_empty(),"source sibling under Outer was moved to the root");
}

#[test]
fn a_named_folder_pushes_and_pops_only_its_own_path_component() {
    let items=parse_bookmark_items(&folder_input(Some("Inner"))).unwrap();
    assert_eq!(path(&items[0]),vec!["Outer","Inner"]);
    assert_eq!(path(&items[1]),vec!["Outer"]);
}

#[test]
fn a_folder_without_a_name_keeps_its_parent_for_the_following_sibling() {
    let items=parse_bookmark_items(&folder_input(None)).unwrap();
    assert_eq!(path(&items[0]),vec!["Outer"]);
    assert_eq!(path(&items[1]),vec!["Outer"]);
}

#[test]
fn an_in_range_i64_timestamp_panics_in_the_public_result_returning_parser() {
    let input=format!(r#"{{"roots":{{"bookmark_bar":{{"children":[{{"type":"url","url":"https://time.test/","date_added":"{}"}}]}}}}}}"#,i64::MIN);
    assert!(std::panic::catch_unwind(||parse_bookmark_items(&input)).is_err());
}

#[test]
fn the_chrome_epoch_offset_control_maps_to_unix_zero() {
    let input=r#"{"roots":{"bookmark_bar":{"children":[{"type":"url","url":"https://time.test/","date_added":"11644473600000000"}]}}}"#;
    let items=parse_bookmark_items(input).unwrap();
    assert_eq!(items[0].created_at_unix_secs,Some(0));
}
