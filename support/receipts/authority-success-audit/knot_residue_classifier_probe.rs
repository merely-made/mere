use std::collections::BTreeSet;
struct Run { freed: Vec<usize>, live: Vec<usize>, frame: usize }
const OUTER_FRAME: usize = usize::MAX;
fn own_blocks(runs: &[Run], baseline: &[&[Run]]) -> BTreeSet<(bool, usize)> {
    let blocks = |run: &Run| -> BTreeSet<(bool, usize)> {
        let outer = |size: usize| if size == run.frame { OUTER_FRAME } else { size };
        let freed = run.freed.iter().map(|&size| (false, outer(size)));
        freed
            .chain(run.live.iter().map(|&size| (true, outer(size))))
            .collect()
    };
    let seen: BTreeSet<(bool, usize)> = baseline
        .iter()
        .flat_map(|runs| runs.iter())
        .flat_map(blocks)
        .collect();
    let mut always = blocks(&runs[0]);
    for run in &runs[1..] {
        always = always.intersection(&blocks(run)).copied().collect();
    }
    always.difference(&seen).copied().collect()
}
fn run(live: Vec<usize>) -> Run {
    Run { freed: vec![], live, frame: 4096 }
}
#[test]
fn persistent_unique_size_is_detected() {
    let host: Vec<_> = (0..5).map(|_| run(vec![64])).collect();
    assert_eq!(own_blocks(&host, &[]), BTreeSet::from([(true, 64)]));
}
#[test]
fn intermittent_unique_copy_is_discarded() {
    let mut host: Vec<_> = (0..5).map(|_| run(vec![])).collect();
    host[4].live.push(64);
    assert!(own_blocks(&host, &[]).is_empty());
}
#[test]
fn persistent_copy_matching_any_baseline_size_is_discarded() {
    let host: Vec<_> = (0..5).map(|_| run(vec![64])).collect();
    let mut baseline: Vec<_> = (0..5).map(|_| run(vec![])).collect();
    baseline[0].live.push(64);
    assert!(own_blocks(&host, &[&baseline]).is_empty());
}
#[test]
fn extra_copy_multiplicity_is_discarded() {
    let host: Vec<_> = (0..5).map(|_| run(vec![64, 64])).collect();
    let baseline: Vec<_> = (0..5).map(|_| run(vec![64])).collect();
    assert!(own_blocks(&host, &[&baseline]).is_empty());
}
