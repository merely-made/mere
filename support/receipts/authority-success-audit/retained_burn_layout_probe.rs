#![allow(dead_code)]
use core::any::Any;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// The fusion backend service is already running, so the change could not
    /// be applied consistently across streams.
    ServiceRunning,
    /// A provider with the same name is already registered.
    DuplicateOptimization {
        /// The conflicting name.
        name: String,
    },
}
type Slot = Box<dyn Any + Send + Sync>;

/// The providers and the fusion-service flag; sealed against changes once the
/// service starts.
struct Registry {
    providers: Vec<(String, Slot)>,
    started: bool,
}

impl Registry {
    fn seeded(defaults: impl FnOnce() -> Vec<(String, Slot)>) -> Self {
        Self {
            providers: defaults(),
            started: false,
        }
    }

    fn register(&mut self, name: String, slot: Slot) -> Result<(), RegistryError> {
        self.ensure_open()?;
        if self.providers.iter().any(|(other, _)| *other == name) {
            return Err(RegistryError::DuplicateOptimization { name });
        }
        self.providers.push((name, slot));
        Ok(())
    }

    fn remove(&mut self, name: &str) -> Result<(), RegistryError> {
        self.ensure_open()?;
        self.providers.retain(|(other, _)| other != name);
        Ok(())
    }

    fn start(&mut self) -> &Self {
        self.started = true;
        self
    }

    fn provider(&self, name: &str) -> Option<&Slot> {
        self.providers
            .iter()
            .find(|(other, _)| other == name)
            .map(|(_, slot)| slot)
    }

    /// Registrations and removals only apply before the fusion service starts.
    fn ensure_open(&self) -> Result<(), RegistryError> {
        if self.started {
            return Err(RegistryError::ServiceRunning);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot() -> Slot {
        Box::new(())
    }

    fn empty() -> Registry {
        Registry::seeded(Vec::new)
    }

    fn seeded() -> Vec<(String, Slot)> {
        vec![("builtin".into(), slot())]
    }

    #[test]
    fn register_then_remove_round_trips() {
        let mut registry = empty();

        registry
            .register("custom".into(), slot())
            .expect("first registration succeeds");
        registry.remove("custom").expect("removal succeeds");

        // The provider is gone, so the same name registers again.
        registry
            .register("custom".into(), slot())
            .expect("re-registration after removal succeeds");
    }

    #[test]
    fn duplicate_names_are_rejected() {
        let mut registry = empty();

        registry.register("custom".into(), slot()).unwrap();
        assert_eq!(
            registry.register("custom".into(), slot()),
            Err(RegistryError::DuplicateOptimization {
                name: "custom".into()
            })
        );
    }

    #[test]
    fn a_started_registry_is_sealed() {
        let mut registry = empty();
        registry.start();

        assert_eq!(
            registry.register("custom".into(), slot()),
            Err(RegistryError::ServiceRunning)
        );
        assert_eq!(
            registry.remove("builtin"),
            Err(RegistryError::ServiceRunning)
        );
    }

    #[test]
    fn provider_lookup_by_name() {
        let mut registry = empty();
        registry.register("custom".into(), slot()).unwrap();

        assert!(registry.provider("custom").is_some());
        assert!(registry.provider("unknown").is_none());
    }

    #[test]
    fn defaults_seed_the_registry() {
        let registry = Registry::seeded(seeded);

        assert!(registry.provider("builtin").is_some());
    }

    #[test]
    fn defaults_are_removable_and_reserve_their_name() {
        let mut registry = Registry::seeded(seeded);

        // A seeded name is held like any other, so it cannot be registered over...
        assert_eq!(
            registry.register("builtin".into(), slot()),
            Err(RegistryError::DuplicateOptimization {
                name: "builtin".into()
            })
        );

        // ...but it is not privileged either: removing it frees the name for a replacement.
        registry.remove("builtin").unwrap();
        assert!(registry.provider("builtin").is_none());
        registry
            .register("builtin".into(), slot())
            .expect("the name is free after removal");
    }
}
fn is_spatial_contiguous(shape: &[usize], strides: &[usize]) -> bool {
    let rank = shape.len();
    let dim_c = rank - 1;

    // Channel must be contiguous for the [(N, H, W), C] reshape to be valid
    if strides[dim_c] != 1 {
        return false;
    }

    for i in (1..dim_c).rev() {
        if strides[i + 1] * shape[i + 1] != strides[i] {
            return false;
        }
    }
    true
}

struct Args{conv_stride_0:usize,conv_stride_1:usize,dilation_0:usize,dilation_1:usize,padding_0:usize,padding_1:usize}
struct Input{height:usize,width:usize}
impl Input{fn shape(&self,dim:usize)->usize{match dim{2=>self.height,3=>self.width,_=>panic!("fixture shape")}}}
fn clamp(v:i32,a:i32,b:i32)->i32{v.clamp(a,b)}
fn clamp_min(v:i32,a:i32)->i32{v.max(a)}
fn direct_ranges(kernel_h:usize,kernel_w:usize,out_y:usize,out_x:usize,input:&Input,args:&Args)->((usize,usize),(usize,usize)){
    let stride_0_i = args.conv_stride_0 as i32;
    let stride_1_i = args.conv_stride_1 as i32;

    let kms_h = (kernel_h * args.dilation_0) as i32 - stride_0_i;
    let kms_w = (kernel_w * args.dilation_1) as i32 - stride_1_i;

    let y_start = ((out_y + args.padding_0) as i32 - kms_h) / stride_0_i;
    let x_start = ((out_x + args.padding_1) as i32 - kms_w) / stride_1_i;

    let y_end = clamp(kms_h + y_start + 1, 0, input.shape(2) as i32) as usize;
    let x_end = clamp(kms_w + x_start + 1, 0, input.shape(3) as i32) as usize;
    let y_start = clamp_min(y_start, 0) as usize;
    let x_start = clamp_min(x_start, 0) as usize;


 ((y_start,y_end),(x_start,x_end))
}
#[cfg(test)]mod layout_cases{
use super::*;
fn source_offsets(shape:&[usize],strides:&[usize])->Vec<usize>{
 let count:usize=shape.iter().product();(0..count).map(|index|{
  let mut rest=index;let mut offset=0;for d in (0..shape.len()).rev(){offset+=(rest%shape[d])*strides[d];rest/=shape[d];}offset
 }).collect()
}
fn flattened_offsets(shape:&[usize],strides:&[usize])->Vec<usize>{
 let c=shape.len()-1;let rows:usize=shape[..c].iter().product();
 (0..rows).flat_map(|row|(0..shape[c]).map(move|col|row*strides[c-1]+col*strides[c])).collect()
}
#[test]fn sliced_batch_gap_is_wrongly_declared_flattenable(){
 // Slice one H row from each of two [2,2,2,1] batch images.
 let shape=[2,1,2,1];let strides=[4,2,1,1];
 assert!(is_spatial_contiguous(&shape,&strides));
 let original=source_offsets(&shape,&strides);let flattened=flattened_offsets(&shape,&strides);
 assert_eq!(original,vec![0,1,4,5]);assert_eq!(flattened,vec![0,1,2,3]);assert_ne!(original,flattened);
 println!("retained reshape_input accepts batch gap: source={original:?}, flattened={flattened:?}");
}
#[test]fn truly_contiguous_batch_control_agrees(){
 let shape=[2,1,2,1];let strides=[2,2,1,1];assert!(is_spatial_contiguous(&shape,&strides));assert_eq!(source_offsets(&shape,&strides),flattened_offsets(&shape,&strides));
}
#[test]fn spatial_gap_control_is_refused(){assert!(!is_spatial_contiguous(&[2,2,2,1],&[8,4,1,1]));}
#[test]fn channel_gap_control_is_refused(){assert!(!is_spatial_contiguous(&[2,2,2,1],&[8,4,2,2]));}
fn args(stride:usize)->Args{Args{conv_stride_0:stride,conv_stride_1:stride,dilation_0:1,dilation_1:1,padding_0:0,padding_1:0}}
#[test]fn stride_wider_than_unit_kernel_misses_valid_origin(){
 let(r,c)=direct_ranges(1,1,0,0,&Input{height:2,width:2},&args(3));
 assert_eq!((r,c),((0,0),(0,0)));
 // For transposed convolution, input(0,0)*weight(0,0) contributes
 // to output(0,0) when stride=3,padding=0,dilation=1,kernel=1.
 assert_eq!(0usize*3+0usize*1,0);assert!(!(r.0..r.1).contains(&0));
 println!("retained direct transpose unit kernel stride3 excludes valid origin: rows={r:?} columns={c:?}");
}
#[test]fn unit_stride_unit_kernel_origin_control(){
 let(r,c)=direct_ranges(1,1,0,0,&Input{height:2,width:2},&args(1));assert!((r.0..r.1).contains(&0)&&(c.0..c.1).contains(&0));
}
#[test]fn unit_stride_three_kernel_origin_control(){
 let(r,c)=direct_ranges(3,3,0,0,&Input{height:2,width:2},&args(1));assert!((r.0..r.1).contains(&0)&&(c.0..c.1).contains(&0));
}
}
