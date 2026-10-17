extern crate cambium;
use cambium_rootstock::{ProducerFrameInfo,ResolvedAppearance};
struct Placement{bounds:[f32;4],scale:f32}
impl Placement{fn physical_size(&self)->[u32;2]{[100,80]}}
struct Fixture{placement:Placement,invalidated:bool}
impl Fixture{fn info(&self,timestamp:Option<std::time::Duration>)->ProducerFrameInfo{
        let info = ProducerFrameInfo {
            logical_size: (self.placement.bounds[2], self.placement.bounds[3]),
            physical_size: self.placement.physical_size(),
            layout_scale: self.placement.scale,
            needs_frame: self.invalidated,
            timestamp,
            appearance: ResolvedAppearance::default(),
        };
info
}}
fn main(){}
