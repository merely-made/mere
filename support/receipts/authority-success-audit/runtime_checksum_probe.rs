#![allow(dead_code)]
struct Function{name:String}struct Candidate{function:Function}struct Set{tunables:Vec<Candidate>}
impl Set{fn checksum_preimage(&self)->String{
        let mut checksum = String::new();
        for tune in &self.tunables {
            checksum += &tune.function.name;
        }

checksum
}}
fn names(values:&[&str])->Set{Set{tunables:values.iter().map(|name|Candidate{function:Function{name:(*name).into()}}).collect()}}
#[test]fn renamed_candidates_with_same_count_share_the_hash_input(){
 let a=names(&["ab","c"]).checksum_preimage();let b=names(&["a","bc"]).checksum_preimage();assert_eq!(a,"abc");assert_eq!(a,b);
}
#[test]fn a_changed_candidate_count_shares_the_hash_input(){
 let a=names(&["abc"]).checksum_preimage();let b=names(&["ab","c"]).checksum_preimage();assert_eq!(a,b);
}
#[test]fn changed_concatenation_control_is_distinct(){assert_ne!(names(&["ab","c"]).checksum_preimage(),names(&["a","bd"]).checksum_preimage());}
