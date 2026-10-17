#![allow(dead_code,unused_variables)]
use std::marker::PhantomData;
type InvalidConfigError=Box<dyn std::fmt::Debug>;
struct FormattedConfigError;impl FormattedConfigError{fn new(f:impl FnOnce()->String)->InvalidConfigError{Box::new(f())}}
struct DeviceProperties;
#[derive(Clone,Copy)]struct Dtype;impl Dtype{fn size_bits(&self)->usize{32}}
#[derive(Clone,Copy)]struct StageMemoryConfig{dtype:Dtype,vector_size:u32,num_planes:u32,elements_per_tile:u32,tiles_row:u32,tiles_col:u32,contiguous_width:u32}
impl StageMemoryConfig{
 fn elements_per_tile(&self)->u32{self.elements_per_tile}
 fn tiles_per_stage(&self)->u32{self.tiles_row*self.tiles_col}
 fn elements_per_stage(&self)->u32{self.elements_per_tile()*self.tiles_per_stage()}
 fn elements_per_tile_along_contiguous_dim(&self)->u32{self.contiguous_width}
 fn tiles_per_stage_along_row(&self)->u32{self.tiles_row}
 fn tiles_per_stage_along_col(&self)->u32{self.tiles_col}
}
#[derive(Clone,Copy)]struct GlobalMemoryConfig{dtype:Dtype,vector_size:usize}
#[derive(Clone,Copy)]enum ReaderMode{Strict,Relaxed}
#[derive(Clone,Copy,PartialEq)]enum StageIdent{Lhs,Rhs}
#[derive(Clone,Copy)]enum InputLoadFlow{MainOnly,LoadOnly}
#[derive(Clone,Copy)]struct GlobalReaderConfig{smem_config:StageMemoryConfig,gmem_config:GlobalMemoryConfig,reader_mode:ReaderMode,plane_dim:u32,stage_ident:StageIdent,input_load_flow:InputLoadFlow}
impl GlobalReaderConfig{fn loading_units_count(&self)->u32{self.plane_dim*self.smem_config.num_planes}fn loading_planes_count(&self)->u32{self.smem_config.num_planes}}
trait TilingOrder{}
struct TestOrder;impl TilingOrder for TestOrder{}
struct OrderedTilingOrder;impl TilingOrder for OrderedTilingOrder{}
struct ContiguousTilingLayout<T>(PhantomData<T>);impl<T:TilingOrder>ContiguousTilingLayout<T>{fn check(_:StageMemoryConfig)->Result<(),InvalidConfigError>{Ok(())}}
fn validate_swizzle_atom_size(_:StageMemoryConfig)->Result<(),InvalidConfigError>{Ok(())}
fn validate_async_barrier(_: &DeviceProperties)->Result<(),InvalidConfigError>{Ok(())}
fn validate_async_copy(_: &DeviceProperties,_a:&Dtype,_b:&Dtype)->Result<(),InvalidConfigError>{Ok(())}
const ASYNC_COPY_WIDTH:u32=128;
trait LoadingValidation{fn validate_with_config(device_props:&DeviceProperties,config:&GlobalReaderConfig)->Result<(),InvalidConfigError>;}
struct sync_partial_cyclic<TO>(PhantomData<TO>);impl<TO:TilingOrder>LoadingValidation for sync_partial_cyclic<TO>{
fn validate_with_config(
        _device_props: &DeviceProperties,
        config: &GlobalReaderConfig,
    ) -> Result<(), InvalidConfigError> {
        if let ReaderMode::Strict = config.reader_mode {
            let vector_size = config.gmem_config.vector_size as u32;
            let num_vectors_per_tile = config.smem_config.elements_per_tile() / vector_size;
            let num_tiles_in_stage = config.smem_config.tiles_per_stage();
            let total_num_vectors = num_tiles_in_stage * num_vectors_per_tile;

            let total_units = config.loading_units_count();
            let jump_length = total_units * vector_size;
            let num_tasks_per_unit = total_num_vectors.div_ceil(total_units);

            let max_id = total_units - 1;
            let max_task_id = num_tasks_per_unit - 1;
            let max_position_base = max_id * vector_size;
            let max_position = max_position_base + max_task_id * jump_length;
            let num_stage_elements = config.smem_config.elements_per_stage();

            if max_position > num_stage_elements {
                return Err(Box::new(
                    "Too many data will be loaded, resulting in out-of-bounds",
                ));
            }
        }

        validate_swizzle_atom_size(config.smem_config)?;
        ContiguousTilingLayout::<TO>::check(config.smem_config)?;

        Ok(())
    }
}
struct async_partial_cyclic<TO>(PhantomData<TO>);impl<TO:TilingOrder>LoadingValidation for async_partial_cyclic<TO>{
fn validate_with_config(
        device_props: &DeviceProperties,
        config: &GlobalReaderConfig,
    ) -> Result<(), InvalidConfigError> {
        let vector_size = ASYNC_COPY_WIDTH / config.smem_config.dtype.size_bits() as u32;
        if let ReaderMode::Strict = config.reader_mode {
            let num_vectors_per_tile = config.smem_config.elements_per_tile() / vector_size;
            let num_tiles_in_stage = config.smem_config.tiles_per_stage();
            let total_num_vectors = num_tiles_in_stage * num_vectors_per_tile;

            let total_units = config.loading_units_count();
            let jump_length = total_units * vector_size;
            let num_tasks_per_unit = total_num_vectors.div_ceil(total_units);

            let max_id = total_units - 1;
            let max_task_id = num_tasks_per_unit - 1;
            let max_position_base = max_id * vector_size;
            let max_position = max_position_base + max_task_id * jump_length;
            let num_stage_elements = config.smem_config.elements_per_stage();

            if max_position > num_stage_elements {
                return Err(Box::new(
                    "Too many data will be loaded, resulting in out-of-bounds",
                ));
            }
        }

        // Needs separate check because copy size may be larger than global vector size
        if !config
            .smem_config
            .elements_per_tile_along_contiguous_dim()
            .is_multiple_of(vector_size)
        {
            return Err(Box::new("Tile size isn't divisible by copy vector size"));
        }

        validate_swizzle_atom_size(config.smem_config)?;
        validate_async_barrier(device_props)?;
        validate_async_copy(
            device_props,
            &config.gmem_config.dtype,
            &config.smem_config.dtype,
        )?;
        ContiguousTilingLayout::<TO>::check(config.smem_config)?;

        Ok(())
    }
}
struct sync_full_cyclic<TO>(PhantomData<TO>);impl<TO:TilingOrder>LoadingValidation for sync_full_cyclic<TO>{
fn validate_with_config(
        _device_props: &DeviceProperties,
        config: &GlobalReaderConfig,
    ) -> Result<(), InvalidConfigError> {
        if let ReaderMode::Strict = config.reader_mode {
            let vector_size = config.gmem_config.vector_size;

            let num_stage_vectors = config.smem_config.elements_per_stage() / vector_size as u32;
            let total_units = config.loading_units_count();

            if !num_stage_vectors.is_multiple_of(total_units) {
                return Err(Box::new(format!(
                "Too many data will be loaded, resulting in out of bounds.
        Try setting vector size and number of planes so that total unit count {total_units:?} divides number of vectors in stage.",
            )));
            }
        }

        validate_swizzle_atom_size(config.smem_config)?;
        ContiguousTilingLayout::<TO>::check(config.smem_config)?;

        Ok(())
    }
}
struct async_full_cyclic<TO>(PhantomData<TO>);impl<TO:TilingOrder>LoadingValidation for async_full_cyclic<TO>{
fn validate_with_config(
        device_props: &DeviceProperties,
        config: &GlobalReaderConfig,
    ) -> Result<(), InvalidConfigError> {
        let vector_size = ASYNC_COPY_WIDTH / config.smem_config.dtype.size_bits() as u32;

        if let ReaderMode::Strict = config.reader_mode {
            let num_stage_vectors = config.smem_config.elements_per_stage() / vector_size;
            let total_units = config.loading_units_count();

            if !num_stage_vectors.is_multiple_of(total_units) {
                return Err(Box::new(format!(
                "Too many data will be loaded, resulting in out of bounds.
        Try setting vector size and number of planes so that total unit count {total_units:?} divides number of vectors in stage.",
            )));
            }
        }

        // Needs separate check because copy size may be larger than global vector size
        if !config
            .smem_config
            .elements_per_tile_along_contiguous_dim()
            .is_multiple_of(vector_size)
        {
            return Err(Box::new("Tile size isn't divisible by copy vector size"));
        }

        validate_swizzle_atom_size(config.smem_config)?;
        validate_async_barrier(device_props)?;
        validate_async_copy(
            device_props,
            &config.gmem_config.dtype,
            &config.smem_config.dtype,
        )?;
        ContiguousTilingLayout::<TO>::check(config.smem_config)?;

        Ok(())
    }
}
struct Ordered;impl LoadingValidation for Ordered{
fn validate_with_config(
        _device_props: &DeviceProperties,
        config: &GlobalReaderConfig,
    ) -> Result<(), InvalidConfigError> {
        if config.stage_ident != StageIdent::Lhs {
            return Err(FormattedConfigError::new(move || {
                "Ordered loading only available on Lhs".to_string()
            }));
        }

        let vector_size = config.gmem_config.vector_size;
        let num_planes = config.loading_planes_count();
        let num_tiles = config.smem_config.tiles_per_stage();

        if !num_tiles.is_multiple_of(num_planes) {
            return Err(FormattedConfigError::new(move || {
                format!(
                    "Number of planes {num_planes:?} must divide number of tiles {num_tiles:?} for ordered loading.",
                )
            }));
        }

        let num_tiles_per_plane = num_tiles / num_planes;
        let num_vectors_per_tile = config.smem_config.elements_per_tile() / vector_size as u32;
        let num_vectors_per_plane = num_vectors_per_tile * num_tiles_per_plane;
        let num_planes = config.loading_planes_count();
        let plane_dim = config.plane_dim;
        let rows_per_plane = config.smem_config.tiles_per_stage_along_row() / num_planes;

        if !num_vectors_per_plane.is_multiple_of(plane_dim) {
            return Err(FormattedConfigError::new(move || {
                format!(
                    "Plane dimension {plane_dim:?} must divide number of vectors per plane {num_vectors_per_plane:?} for ordered loading.",
                )
            }));
        }

        let tile_count_col = config.smem_config.tiles_per_stage_along_col();
        if num_tiles_per_plane != rows_per_plane * tile_count_col {
            return Err(FormattedConfigError::new(move || {
                format!(
                    "Number of tiles per plane {num_tiles_per_plane:?} must equal rows_per_plane {rows_per_plane:?} times cols {tile_count_col:?} for ordered loading.",
                )
            }));
        }

        validate_swizzle_atom_size(config.smem_config)?;
        ContiguousTilingLayout::<OrderedTilingOrder>::check(config.smem_config)?;

        Ok(())
    }
}

fn config(vectors:u32)->GlobalReaderConfig{GlobalReaderConfig{smem_config:StageMemoryConfig{dtype:Dtype,vector_size:4,num_planes:1,elements_per_tile:4,tiles_row:1,tiles_col:vectors,contiguous_width:4},gmem_config:GlobalMemoryConfig{dtype:Dtype,vector_size:4},reader_mode:ReaderMode::Strict,plane_dim:32,stage_ident:StageIdent::Lhs,input_load_flow:InputLoadFlow::MainOnly}}
fn last_start(c:&GlobalReaderConfig)->u32{let v=4;let units=c.loading_units_count();let total=c.smem_config.elements_per_stage()/v;let tasks=total.div_ceil(units);(units-1)*v+(tasks-1)*units*v}
#[test]fn sync_partial_strict_admits_start_exactly_at_stage_end(){let c=config(63);assert_eq!(last_start(&c),c.smem_config.elements_per_stage());assert!(sync_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_ok());}
#[test]fn async_partial_strict_admits_start_exactly_at_stage_end(){let c=config(63);assert_eq!(last_start(&c),c.smem_config.elements_per_stage());assert!(async_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_ok());}
#[test]fn sync_full_rejects_same_imbalanced_workload(){assert!(sync_full_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(63)).is_err());}
#[test]fn async_full_rejects_same_imbalanced_workload(){assert!(async_full_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(63)).is_err());}
#[test]fn sync_partial_balanced_control(){assert!(sync_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(64)).is_ok());}
#[test]fn async_partial_balanced_control(){assert!(async_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&config(64)).is_ok());}
#[test]fn sync_partial_further_overrun_control_is_rejected(){let c=config(62);assert!(last_start(&c)>c.smem_config.elements_per_stage());assert!(sync_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_err());}
#[test]fn async_partial_further_overrun_control_is_rejected(){let c=config(62);assert!(last_start(&c)>c.smem_config.elements_per_stage());assert!(async_partial_cyclic::<TestOrder>::validate_with_config(&DeviceProperties,&c).is_err());}
fn ordered_config(flow:InputLoadFlow)->GlobalReaderConfig{let mut c=config(1);c.smem_config.elements_per_tile=256;c.smem_config.contiguous_width=16;c.input_load_flow=flow;c}
#[test]fn ordered_main_flow_control(){assert!(Ordered::validate_with_config(&DeviceProperties,&ordered_config(InputLoadFlow::MainOnly)).is_ok());}
#[test]fn ordered_loader_validator_alone_also_accepts_load_only(){assert!(Ordered::validate_with_config(&DeviceProperties,&ordered_config(InputLoadFlow::LoadOnly)).is_ok());}
