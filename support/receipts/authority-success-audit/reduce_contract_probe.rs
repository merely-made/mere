#![allow(dead_code,private_bounds,private_interfaces)]
#[derive(Debug)] enum ReduceError { InvalidAxis{axis:usize,rank:usize},ReduceAxisTooSmall{axis_length:usize,k:usize},MismatchOutputShape{expected_shape:Vec<usize>,output_shape:Vec<usize>}, Validation{details:&'static str} }
trait Numeric:Copy+PartialOrd {} impl Numeric for f32{} impl Numeric for u32{}
#[derive(Clone,Copy)]struct Vector<N>(N);
impl<N:Numeric> Vector<N>{fn vector_size(self)->usize{1} fn extract(self,_:usize)->N{self.0} fn unwrap(self)->Self{self} fn greater_than(&self,o:&Self)->Vector<bool>{Vector(self.0>o.0)} fn less_than(&self,o:&Self)->Vector<bool>{Vector(self.0<o.0)} fn equal(&self,o:&Self)->Vector<bool>{Vector(self.0==o.0)}}
trait Comptime{fn comptime(self)->Self;} impl Comptime for usize{fn comptime(self)->Self{self}}
type Array<T> = Vec<T>;
enum Value<T>{None,Single(T),Multiple(Array<T>)}
impl<T> Value<T>{fn multiple_mut(&mut self)->&mut Array<T>{match self{Self::Multiple(a)=>a,_=>panic!("expected multiple")}}}
fn select_many<N>(p:Vector<bool>,a:Vector<N>,b:Vector<N>)->Vector<N>{if p.0{a}else{b}}
fn validate_axis(rank: usize, axis: usize) -> Result<(), ReduceError> {
    if axis > rank {
        return Err(ReduceError::InvalidAxis { axis, rank });
    }
    Ok(())
}
fn validate_shapes(
    input_shape: &[usize],
    output_shape: &[usize],
    axis: usize,
    k: Option<usize>,
) -> Result<(), ReduceError> {
    let mut expected_shape = input_shape.to_vec();
    let k = k.unwrap_or(1);

    if expected_shape[axis] < k {
        return Err(ReduceError::ReduceAxisTooSmall {
            axis_length: expected_shape[axis],
            k,
        });
    }

    expected_shape[axis] = k;
    if output_shape != expected_shape {
        return Err(ReduceError::MismatchOutputShape {
            expected_shape,
            output_shape: output_shape.to_vec(),
        });
    }
    Ok(())
}
fn reaches<N: Numeric>(item: Vector<N>, kth: Vector<N>) -> bool {
    let mut any = false;
    for i in 0..item.vector_size().comptime() {
        if !(item.extract(i) < kth.extract(i)) {
            any = true;
        }
    }
    any
}
fn topk_insert<N: Numeric>(
    elements: &mut Array<Vector<N>>,
    coordinates: &mut Value<Vector<u32>>,
    insert_val: Vector<N>,
    insert_coord: &Value<Vector<u32>>,
    k: usize,
) {
    if reaches(insert_val, elements[k - 1]) {
        let mut insert_val = insert_val;

        match insert_coord {
            Value::None => {
                for j in 0..k {
                    let to_keep = elements[j].greater_than(&insert_val);
                    let next_val = select_many(to_keep, insert_val, elements[j]);
                    elements[j] = select_many(to_keep, elements[j], insert_val);
                    insert_val = next_val;
                }
            },
            Value::Single(coord) => {
                let mut insert_coord = coord.unwrap();
                let coords = coordinates.multiple_mut();

                for j in 0..k {
                    let to_keep = select_many(
                        elements[j].equal(&insert_val),
                        coords[j].less_than(&insert_coord),
                        elements[j].greater_than(&insert_val),
                    );

                    let next_val = select_many(to_keep, insert_val, elements[j]);
                    elements[j] = select_many(to_keep, elements[j], insert_val);
                    insert_val = next_val;

                    let next_coord = select_many(to_keep, insert_coord, coords[j]);
                    coords[j] = select_many(to_keep, coords[j], insert_coord);
                    insert_coord = next_coord;
                }
            },
            Value::Multiple(_) => panic!("a top-k candidate carries at most one coordinate"),
        }
    }
}
fn unit_guard(working_cubes:usize,launched_cubes:usize,blueprint:UnitBlueprint)->Result<(),ReduceError>{if working_cubes != launched_cubes && blueprint.unit_idle.is_enabled() {
                    return Err(ReduceError::Validation {
                        details: "Too many units launched for the problem causing OOD, but `unit_idle` is off.",
                    });
                }Ok(())}
fn cube_guard(blueprint:CubeBlueprint,cube_dim:CubeDim)->Result<(),ReduceError>{if blueprint.num_shared_accumulators != cube_dim.x as usize {
                        return Err(ReduceError::Validation {
                            details: "Num accumulators should match cube_dim.x",
                        });
                    }Ok(())}

#[derive(Clone,Copy)]struct Idle(bool);impl Idle{fn is_enabled(self)->bool{self.0}}
struct UnitBlueprint{unit_idle:Idle}struct CubeBlueprint{num_shared_accumulators:usize}struct CubeDim{x:u32,y:u32}
fn validate(input:&[usize],output:&[usize],axis:usize,k:Option<usize>)->Result<(),ReduceError>{validate_axis(input.len(),axis)?;validate_shapes(input,output,axis,k)}
#[test]fn axis_equal_rank_panics_after_acceptance(){assert!(validate_axis(2,2).is_ok());assert!(std::panic::catch_unwind(||validate(&[2,3],&[2,1],2,None)).is_err());}
#[test]fn rank_zero_axis_zero_panics_after_acceptance(){assert!(std::panic::catch_unwind(||validate(&[],&[],0,None)).is_err());}
#[test]fn axis_greater_rank_returns_error_control(){assert!(matches!(validate(&[2,3],&[2,1],3,None),Err(ReduceError::InvalidAxis{..})));}
#[test]fn legal_axis_control(){assert!(validate(&[2,3],&[2,1],1,None).is_ok());}
#[test]fn zero_k_is_admitted(){assert!(validate(&[2,3],&[2,0],1,Some(0)).is_ok());}
fn fold(row:&[f32],k:usize,with_coords:bool)->(Vec<f32>,Option<Vec<u32>>){
    // Same finite seed as TopK::null_accumulator; width one, no Cube DSL/backend.
    let mut elements=vec![Vector(f32::MIN);k];
    let mut coords=if with_coords{Value::Multiple(vec![Vector(u32::MAX);k])}else{Value::None};
    for (i,&x) in row.iter().enumerate(){let c=if with_coords{Value::Single(Vector(i as u32))}else{Value::None};topk_insert(&mut elements,&mut coords,Vector(x),&c,k);}
    (elements.into_iter().map(|v|v.0).collect(),match coords{Value::Multiple(v)=>Some(v.into_iter().map(|v|v.0).collect()),_=>None})
}
#[test]fn topk_negative_infinity_is_rejected_by_finite_seed(){let(v,c)=fold(&[f32::NEG_INFINITY],1,true);assert_eq!(v,vec![f32::MIN]);assert_eq!(c,Some(vec![u32::MAX]));}
#[test]fn topk_values_only_also_retains_seed(){assert_eq!(fold(&[f32::NEG_INFINITY],1,false).0,vec![f32::MIN]);}
#[test]fn topk_mixed_finite_negative_infinity_keeps_invalid_second_slot(){let(v,c)=fold(&[3.,f32::NEG_INFINITY],2,true);assert_eq!(v,vec![3.,f32::MIN]);assert_eq!(c,Some(vec![0,u32::MAX]));}
#[test]fn seed_equal_coordinate_control(){let(v,c)=fold(&[f32::MIN,f32::MIN],2,true);assert_eq!(v,vec![f32::MIN,f32::MIN]);assert_eq!(c,Some(vec![0,1]));}
#[test]fn finite_sort_control(){let(v,c)=fold(&[1.,8.,3.],2,true);assert_eq!(v,vec![8.,3.]);assert_eq!(c,Some(vec![1,2]));}
#[test]fn zero_k_insert_panics(){assert!(std::panic::catch_unwind(||fold(&[1.],0,true)).is_err());}
#[test]fn topk_nan_order_changes_winner(){assert_eq!(fold(&[f32::NAN,1.],1,true).0,vec![1.]);assert!(fold(&[1.,f32::NAN],1,true).0[0].is_nan());}
#[test]fn forced_unit_enabled_idle_rejected_on_extra_cubes(){assert!(unit_guard(1,2,UnitBlueprint{unit_idle:Idle(true)}).is_err());}
#[test]fn forced_unit_disabled_idle_admits_extra_cubes(){assert!(unit_guard(1,2,UnitBlueprint{unit_idle:Idle(false)}).is_ok());}
#[test]fn forced_unit_partial_cube_guard_never_compares_units(){let units=1usize;let cube_units=32usize;let cubes=units.div_ceil(cube_units);assert!(unit_guard(cubes,cubes,UnitBlueprint{unit_idle:Idle(false)}).is_ok());assert!(cubes*cube_units>units);}
#[test]fn forced_cube_rejects_inferred_per_plane_slot_count(){let d=CubeDim{x:32,y:4};assert!(cube_guard(CubeBlueprint{num_shared_accumulators:d.y as usize},d).is_err());}
#[test]fn forced_cube_accepts_lane_count_instead_of_plane_count(){let d=CubeDim{x:32,y:4};assert!(cube_guard(CubeBlueprint{num_shared_accumulators:d.x as usize},d).is_ok());}
