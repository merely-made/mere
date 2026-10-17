use std::collections::HashMap;
use uuid::Uuid;
use chirograph::PortableCardV1;
pub fn add_node_facets(
    cards: &mut HashMap<String, PortableCardV1>,
    facets: &kernel::graph::NodeFacetStore,
) -> Result<(), String> {
    for (key, card) in cards {
        let id = Uuid::parse_str(key).map_err(|e| e.to_string())?;
        if let Some(facets) = facets.facets_of(&id) {
            let mut values = Vec::new();
            for (_, value) in facets.iter() {
                record_values("", value, &mut values);
            }
            values.append(&mut card.values);
            card.values = values;
        }
    }
    Ok(())
}
fn record_values(
    prefix: &str,
    value: &serde_json::Value,
    values: &mut Vec<chirograph::CardValueV1>,
) {
    match value {
        serde_json::Value::Object(fields) => {
            for (label, value) in fields {
                let label = label.replace('_', " ");
                let label = if prefix.is_empty() {
                    label
                } else {
                    format!("{prefix} · {label}")
                };
                record_values(&label, value, values);
            }
        },
        serde_json::Value::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                record_values(&format!("{prefix} {}", index + 1), value, values);
            }
        },
        serde_json::Value::Null => {},
        value => values.push(chirograph::CardValueV1 {
            label: prefix.into(),
            value: value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string()),
        }),
    }
}

fn card() -> PortableCardV1 { PortableCardV1 { title: "Selected node".into(),
 values: vec![chirograph::CardValueV1 {label:"Address".into(),value:"https://owner.test/selected".into()}],badges:vec![],media:vec![] } }
fn main() {
 let id=Uuid::from_u128(1);let key=id.to_string();
 let first=|cards:&HashMap<String,PortableCardV1>| cards[&key].values.iter().find(|v|v.label=="Address").unwrap().value.clone();
 let mut cards=HashMap::from([(key.clone(),card())]);
 add_node_facets(&mut cards,&kernel::graph::NodeFacetStore::new()).unwrap();
 assert_eq!(first(&cards),"https://owner.test/selected");
 println!("PASS no-facet control keeps owner address first");
 let mut facets=kernel::graph::NodeFacetStore::new();
 facets.set(id,chartulary::FacetId::new("product.record"),serde_json::json!({"Address":"https://other.test/facet"}),&chartulary::AcceptAll).unwrap();
 let mut cards=HashMap::from([(key.clone(),card())]);add_node_facets(&mut cards,&facets).unwrap();
 assert_eq!(first(&cards),"https://other.test/facet");
 assert!(cards[&key].values.iter().any(|v|v.label=="Address"&&v.value=="https://owner.test/selected"));
 println!("PASS facet Address shadows the still-present owner address for first-label consumers");
 let mut facets=kernel::graph::NodeFacetStore::new();
 facets.set(id,chartulary::FacetId::new("product.record"),serde_json::json!({"address":"https://other.test/facet"}),&chartulary::AcceptAll).unwrap();
 let mut cards=HashMap::from([(key.clone(),card())]);add_node_facets(&mut cards,&facets).unwrap();
 assert_eq!(first(&cards),"https://owner.test/selected");
 println!("PASS lowercase address control does not collide with the exact Address label");
}
