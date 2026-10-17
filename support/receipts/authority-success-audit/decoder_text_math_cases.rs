#[cfg(test)] mod audit {
    use super::*;
    fn provider() -> DecoderProvider {
        let bytes = br#"{"version":"1.0","truncation":null,"padding":null,"added_tokens":[],"normalizer":null,"pre_tokenizer":null,"post_processor":null,"decoder":{"type":"Sequence","decoders":[{"type":"ByteFallback"},{"type":"Fuse"}]},"model":{"type":"WordLevel","vocab":{"<0xC3>":0,"<0xA9>":1,"x":2,"[UNK]":3},"unk_token":"[UNK]"}}"#;
        DecoderProvider { tokenizer: Tokenizer::from_bytes(bytes).unwrap(), add_special_tokens: false }
    }
    fn observed(ids: &[u32], stop: Vec<String>) -> (DecoderGeneration, Vec<String>) {
        let p = provider(); let request = GenerationRequest { stop, ..Default::default() };
        let mut fragments = Vec::new(); let mut callback = |s: &str| { fragments.push(s.to_string()); ControlFlow::Continue(()) };
        let mut token_callback = |_:u32| {};
        let mut observer = StreamObserver::new(&p, &request, &mut callback, &mut token_callback);
        for &id in ids { if observer.observe(id).is_break() { break; } }
        let result = observer.finish(ids.to_vec(), false).unwrap(); (result, fragments)
    }
    #[test] fn real_byte_fallback_revises_partial_unicode() {
        let p=provider(); assert_eq!(p.decode(&[0]).unwrap(), "�");
        assert_eq!(p.decode(&[0,1]).unwrap(), "é"); assert_eq!(p.decode(&[0,1,2]).unwrap(), "éx");
    }
    #[test] fn split_utf8_stream_returns_replacement_and_latches() {
        let (result, fragments)=observed(&[0,1,2],vec![]);
        assert_eq!(result.text,"�"); assert_eq!(fragments,vec!["�"]);
        assert_ne!(result.text,provider().decode(&result.token_ids).unwrap());
    }
    #[test] fn ascii_control_and_split_stop_work() {
        let (result, fragments)=observed(&[2,2,2],vec![]);
        assert_eq!(result.text,"xxx"); assert_eq!(fragments,vec!["x","x","x"]);
        let (result, fragments)=observed(&[2,2,2],vec!["xx".into()]);
        assert_eq!(result.text,""); assert!(fragments.is_empty());
    }
    #[test] fn unicode_stop_is_bypassed_after_partial_decode() {
        let (result, fragments)=observed(&[0,1,2],vec!["é".into()]);
        assert_eq!(result.text,"�"); assert_eq!(fragments,vec!["�"]);
        // Correct final decode begins with the requested stop sequence.
        assert!(provider().decode(&result.token_ids).unwrap().starts_with("é"));
    }
    #[test] fn accepted_subnormal_temperature_selects_the_wrong_token() {
        let t=1e-40_f32; assert!(t.is_finite() && t>0.0);
        for top_p in [None,Some(0.9)] {
            let mut s=sample::Sampler::new(t,top_p,7);
            assert_eq!(s.sample(&[2.0,3.0]),0);
            let mut control=sample::Sampler::new(0.01,top_p,7);
            assert_eq!(control.sample(&[2.0,3.0]),1);
        }
    }
    #[test] fn nan_expected_logit_is_ranked_first() {
        assert_eq!(expected_token_rank(&[10.0,1.0,5.0],1),3);
        assert_eq!(expected_token_rank(&[10.0,f32::NAN,5.0],1),1);
        assert_eq!(expected_token_rank(&[f32::NAN;3],1),1);
    }
}
