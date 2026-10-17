use embed::{VectorIndex, SparseIndex, SparseVector, SimilarityMetric, StubEmbeddingProvider, LexicalEmbeddingProvider};
use embed::search::SemanticSearch;
#[test]
fn dense_facade_accepts_a_metric_disagreement_and_its_reads_disagree() {
    let mut index=VectorIndex::new(2,SimilarityMetric::Euclidean);
    index.insert(1u32,vec![10.0,0.0]).unwrap(); index.insert(2,vec![1.0,1.0]).unwrap();
    let search=SemanticSearch::with_index(StubEmbeddingProvider::new(2).unwrap(),index).unwrap();
    assert_eq!(search.metric(),SimilarityMetric::Cosine);
    assert_eq!(search.index().metric(),SimilarityMetric::Euclidean);
    assert_eq!(search.nearest(&[1.0,0.0],1).unwrap()[0].0,2);
    let scores=search.scores(&[1.0,0.0]);
    assert!(scores.iter().find(|(k,_)|*k==1).unwrap().1>scores.iter().find(|(k,_)|*k==2).unwrap().1);
}
#[test]
fn sparse_facade_accepts_the_same_metric_disagreement() {
    let mut index=SparseIndex::new(2,SimilarityMetric::Euclidean);
    index.insert(1u32,SparseVector::from_dense(&[10.0,0.0])).unwrap();
    index.insert(2,SparseVector::from_dense(&[1.0,1.0])).unwrap();
    let search=SemanticSearch::with_sparse_index(LexicalEmbeddingProvider::new(2).unwrap(),index).unwrap();
    assert_eq!(search.metric(),SimilarityMetric::Cosine);
    assert_eq!(search.sparse_index().unwrap().metric(),SimilarityMetric::Euclidean);
    assert_eq!(search.nearest(&[1.0,0.0],1).unwrap()[0].0,2);
    let scores=search.scores(&[1.0,0.0]);
    assert!(scores.iter().find(|(k,_)|*k==1).unwrap().1>scores.iter().find(|(k,_)|*k==2).unwrap().1);
}
#[test]
fn deserialize_bypasses_sparse_pair_admission_and_to_dense_panics() {
    assert!(SparseVector::from_pairs(2,vec![(7,1.0)]).is_err());
    let invalid:SparseVector=serde_json::from_str(r#"{"dimensions":2,"entries":[[7,1.0]]}"#).unwrap();
    assert!(std::panic::catch_unwind(||invalid.to_dense()).is_err());
}
#[test]
fn deserialize_bypasses_outer_index_dimensions() {
    let mut valid=VectorIndex::<u32>::new(2,SimilarityMetric::Cosine);
    assert!(valid.insert(1,vec![1.0]).is_err());
    let invalid:VectorIndex<u32>=serde_json::from_str(r#"{"dimensions":2,"metric":"Cosine","entries":{"1":[1.0]}}"#).unwrap();
    assert_eq!(invalid.get(&1).unwrap().len(),1);
    assert_eq!(invalid.nearest(&[1.0,0.0],1).unwrap()[0],(1,1.0));
}
#[test]
fn admitted_nan_scores_make_the_claimed_total_comparator_cyclic() {
    use std::cmp::Ordering::Less;
    let mut index=VectorIndex::<u32>::new(1,SimilarityMetric::DotProduct);
    index.insert(2,vec![f32::NAN]).unwrap();
    let a=(1u32,1.0); let b=(2u32,f32::NAN); let c=(3u32,2.0);
    assert_eq!(embed::index::rank_order(SimilarityMetric::DotProduct,&a,&b),Less);
    assert_eq!(embed::index::rank_order(SimilarityMetric::DotProduct,&b,&c),Less);
    assert_eq!(embed::index::rank_order(SimilarityMetric::DotProduct,&c,&a),Less);
}
#[test]
fn zero_neighbour_affinity_still_connects_equal_vectors() {
    let mut index=VectorIndex::<u32>::new(2,SimilarityMetric::Cosine);
    for key in [0,1] {index.insert(key,vec![1.0,0.0]).unwrap();}
    assert_eq!(embed::affinity::affinity_pairs(&index,0,0.5).unwrap().len(),1);
}
#[test]
fn dot_affinity_keeps_two_neighbours_when_only_one_was_requested() {
    let mut index=VectorIndex::<u32>::new(1,SimilarityMetric::DotProduct);
    for (key,value) in [(0,1.0),(1,2.0),(2,3.0)] {index.insert(key,vec![value]).unwrap();}
    let pairs=embed::affinity::affinity_pairs(&index,1,0.5).unwrap();
    assert_eq!(pairs.len(),3); // Includes low->middle as well as both ->high.
    assert!(pairs.iter().any(|(a,b,_)|(*a==0&&*b==1)||(*a==1&&*b==0)));
}
