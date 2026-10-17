use std::ops::ControlFlow; use tokenizers::Tokenizer;
#[path="C:/Users/mark_/Code/repos/mere/crates/intel/esp/src/infer/provider.rs"] mod portable;
use portable::{InferError,GenerationRequest};
#[path="C:/Users/mark_/Code/repos/mere/crates/intel/esp/src/infer/decoder/sample.rs"] mod sample;
// Fixture owns only the real tokenizer; no decoder model is compiled.
struct DecoderProvider { tokenizer: Tokenizer, add_special_tokens: bool }
impl DecoderProvider {
    fn decode(&self, ids: &[u32]) -> Result<String, InferError> {
        self.tokenizer
            // A declared template's stop strings can themselves be special
            // tokens. Keep them visible to the stop matcher in literal mode.
            .decode(ids, self.add_special_tokens)
            .map_err(|e| InferError::Backend(format!("detokenize: {e}")))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecoderGeneration {
    /// Complete emitted text, identical to the return from
    /// [`InferenceProvider::generate_streaming`].
    pub text: String,
    /// Generated token ids, excluding the prompt and EOS.
    pub token_ids: Vec<u32>,
    /// Number of non-empty detokenized fragments delivered to the caller.
    pub emitted_fragments: usize,
    /// Whether the caller's fragment callback requested an early stop.
    pub stopped_by_callback: bool,
    /// Whether an async host cancelled before the next token was emitted.
    pub cooperatively_cancelled: bool,
}
struct StreamObserver<'a> {
    provider: &'a DecoderProvider,
    request: &'a GenerationRequest,
    on_fragment: &'a mut dyn FnMut(&str) -> ControlFlow<()>,
    on_generated_token: &'a mut dyn FnMut(u32),
    generated_ids: Vec<u32>,
    emitted: String,
    emitted_fragments: usize,
    stopped_by_callback: bool,
    stopped_by_string: bool,
    decode_error: Option<InferError>,
}

impl Drop for StreamObserver<'_> {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.generated_ids.zeroize();
        self.emitted.zeroize();
    }
}

impl<'a> StreamObserver<'a> {
    fn new(
        provider: &'a DecoderProvider,
        request: &'a GenerationRequest,
        on_fragment: &'a mut dyn FnMut(&str) -> ControlFlow<()>,
        on_generated_token: &'a mut dyn FnMut(u32),
    ) -> Self {
        Self {
            provider,
            request,
            on_fragment,
            on_generated_token,
            generated_ids: Vec::new(),
            emitted: String::new(),
            emitted_fragments: 0,
            stopped_by_callback: false,
            stopped_by_string: false,
            decode_error: None,
        }
    }

    fn observe(&mut self, token: u32) -> ControlFlow<()> {
        (self.on_generated_token)(token);
        self.generated_ids.push(token);
        let full = match self.provider.decode(&self.generated_ids) {
            Ok(full) => full,
            Err(error) => {
                self.decode_error = Some(error);
                return ControlFlow::Break(());
            },
        };
        // Hold back non-prefix decodes (mid-byte BPE boundary); the text
        // arrives with a later token.
        if !full.starts_with(&self.emitted) || full.len() == self.emitted.len() {
            return ControlFlow::Continue(());
        }
        let stop_at = self
            .request
            .stop
            .iter()
            .filter(|stop| !stop.is_empty())
            .filter_map(|stop| full.find(stop))
            .min();
        self.stopped_by_string = stop_at.is_some();
        // Hold a suffix which could become a stop across later fragments.
        // Every prefix length is a UTF-8 boundary; content is never sliced
        // through a character.
        let withheld = self
            .request
            .stop
            .iter()
            .flat_map(|stop| {
                stop.char_indices()
                    .map(|(index, _)| index)
                    .filter(|index| *index > 0)
                    .filter(|index| full.ends_with(&stop[..*index]))
            })
            .max()
            .unwrap_or(0);
        let end = stop_at.unwrap_or(full.len() - withheld);
        if end > self.emitted.len() {
            let delta = &full[self.emitted.len()..end];
            self.stopped_by_callback = (self.on_fragment)(delta).is_break();
            self.emitted_fragments += 1;
            self.emitted = full[..end].to_string();
        }
        if self.stopped_by_callback || self.stopped_by_string {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn finish(
        mut self,
        token_ids: Vec<u32>,
        cooperatively_cancelled: bool,
    ) -> Result<DecoderGeneration, InferError> {
        if let Some(error) = self.decode_error.take() {
            return Err(error);
        }
        // At a normal token limit an incomplete stop prefix is ordinary text.
        // A cancellation never flushes held bytes after the stop boundary.
        if !self.stopped_by_callback && !self.stopped_by_string && !cooperatively_cancelled {
            let full = self.provider.decode(&self.generated_ids)?;
            if full.starts_with(&self.emitted) && full.len() > self.emitted.len() {
                self.stopped_by_callback =
                    (self.on_fragment)(&full[self.emitted.len()..]).is_break();
                self.emitted_fragments += 1;
                self.emitted = full;
            }
        }
        Ok(DecoderGeneration {
            text: std::mem::take(&mut self.emitted),
            token_ids,
            emitted_fragments: self.emitted_fragments,
            stopped_by_callback: self.stopped_by_callback,
            cooperatively_cancelled,
        })
    }
}
pub fn expected_token_rank(logits: &[f32], expected_id: usize) -> usize {
    1 + logits
        .iter()
        .filter(|&&logit| logit > logits[expected_id])
        .count()
}

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
