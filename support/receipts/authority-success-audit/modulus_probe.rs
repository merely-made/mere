use modulus::{BrickMap, BrickProjectionRevision, BrickTrace};

#[test]
fn public_material_lookup_wraps_distant_voxels_to_an_occupied_key() {
    let brick = [7u8;512];
    let map = BrickMap::from_keys(BrickProjectionRevision(1), [[0,0,0]], |_| Some(brick.as_slice())).unwrap();
    assert_eq!(map.material_at([0,0,0]), 7);
    assert_eq!(map.material_at([8,0,0]), 0);
    for x in [-524_288,524_288] {
        assert_eq!(map.material_at([x,0,0]), 7);
        assert_eq!(map.trace([x as f32+0.5,10.0,0.5], [0.0,-1.0,0.0], 20.0), BrickTrace::Clear);
    }
    println!("Public lookup at +/-524288 aliases key0 to material7; nearby outside x8 is air, and actual public trace at the aliased coordinates is Clear.");
}

#[test]
fn exact_parallel_threshold_is_clipped_as_moving_but_never_crosses_a_voxel() {
    let mut brick = [0u8;512];
    for y in 0..8 { for z in 0..8 { for x in 4..8 { brick[(y*8+z)*8+x]=9; } } }
    let map = BrickMap::from_keys(BrickProjectionRevision(1), [[0,0,0]], |_| Some(brick.as_slice())).unwrap();
    let eye = [0.5,4.5,4.5];
    let threshold = 1.0e-6f32;
    let above = f32::from_bits(threshold.to_bits()+1);
    let edge = map.trace(eye,[threshold,0.0,0.0],10_000_000.0);
    assert_eq!(edge,BrickTrace::Clear);
    for direction in [above,1.0] {
        let found=map.trace(eye,[direction,0.0,0.0],10.0/direction);
        let BrickTrace::Hit(hit)=found else { panic!("control should hit: {found:?}") };
        assert_eq!(hit.voxel,[4,4,4]); assert_eq!(hit.material,9);
    }
    println!("At exact1e-6 direction trace is Clear; the next representable value above it and unit-direction control hit voxel[4,4,4]. Clip uses < threshold; crossing uses > threshold.");
}
