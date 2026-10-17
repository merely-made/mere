#![allow(dead_code,private_interfaces,non_camel_case_types)]
extern crate self as cubecl_core;
extern crate self as half;
pub type f16=u16;
pub mod ir {
 pub type Id=u16;
 #[derive(Debug,Clone,PartialEq)]pub enum ConstantValue{UInt(u64),Int(i64),Float(f64),Bool(bool)}
 impl std::fmt::Display for ConstantValue{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{match self {Self::UInt(v)=>write!(f,"{v}"),Self::Int(v)=>write!(f,"{v}"),Self::Float(v)=>write!(f,"{v}"),Self::Bool(v)=>write!(f,"{v}")}}}
}
mod compiler {pub mod wgsl {
 mod base {
use crate::compiler::wgsl::Item::Scalar;
use cubecl_core::ir::{ConstantValue, Id};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    GlobalInputArray(Id, Item),
    GlobalOutputArray(Id, Item),
    GlobalScalar(Id, Elem),
    Constant(ConstantValue, Item),
    LocalMut {
        id: Id,
        item: Item,
    },
    LocalConst {
        id: Id,
        item: Item,
    },
    Named {
        name: String,
        item: Item,
        is_array: bool,
    },
    // TODO: Potential cleanup, seems that variable is not used at all
    LocalScalar {
        id: Id,
        elem: Elem,
    },
    SharedArray(Id, Item, u32),
    SharedValue(Id, Item),
    ConstantArray(Id, Item, u32),
    LocalArray(Id, Item, u32),
    Id,
    LocalInvocationIndex,
    LocalInvocationIdX,
    LocalInvocationIdY,
    LocalInvocationIdZ,
    WorkgroupId,
    WorkgroupIdX,
    WorkgroupIdY,
    WorkgroupIdZ,
    GlobalInvocationIdX,
    GlobalInvocationIdY,
    GlobalInvocationIdZ,
    WorkgroupSize,
    WorkgroupSizeX,
    WorkgroupSizeY,
    WorkgroupSizeZ,
    NumWorkgroups,
    NumWorkgroupsX,
    NumWorkgroupsY,
    NumWorkgroupsZ,
    SubgroupSize,
    SubgroupId,
    SubgroupInvocationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Elem {
    F16,
    F32,
    F64,
    AtomicF32,
    I32,
    I64,
    AtomicI32,
    U32,
    U64,
    AtomicU32,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Item {
    Vec4(Elem),
    Vec3(Elem),
    Vec2(Elem),
    Scalar(Elem),
}

#[derive(Debug, Clone)]
pub struct IndexedVariable {
    var: Variable,
    index: usize,
}

impl Variable {
    pub fn is_always_scalar(&self) -> bool {
        match self {
            Variable::GlobalScalar(_, _) => true,
            Variable::Constant(_, _) => true,
            Variable::LocalScalar { .. } => true,
            Variable::Id => true,
            Variable::LocalInvocationIndex => true,
            Variable::LocalInvocationIdX => true,
            Variable::LocalInvocationIdY => true,
            Variable::LocalInvocationIdZ => true,
            Variable::GlobalInputArray(_, _) => false,
            Variable::GlobalOutputArray(_, _) => false,
            Variable::SharedArray(_, _, _) => false,
            Variable::SharedValue(_, _) => false,
            Variable::ConstantArray(_, _, _) => false,
            Variable::LocalArray(_, _, _) => false,
            Variable::LocalMut { .. } => false,
            Variable::LocalConst { .. } => false,
            Variable::Named { .. } => false,
            Variable::WorkgroupIdX => true,
            Variable::WorkgroupIdY => true,
            Variable::WorkgroupIdZ => true,
            Variable::GlobalInvocationIdX => true,
            Variable::GlobalInvocationIdY => true,
            Variable::GlobalInvocationIdZ => true,
            Variable::WorkgroupSizeX => true,
            Variable::WorkgroupSizeY => true,
            Variable::WorkgroupSizeZ => true,
            Variable::NumWorkgroupsX => true,
            Variable::NumWorkgroupsY => true,
            Variable::NumWorkgroupsZ => true,
            Variable::WorkgroupId => true,
            Variable::WorkgroupSize => true,
            Variable::NumWorkgroups => true,
            Variable::SubgroupSize => true,
            Variable::SubgroupId => true,
            Variable::SubgroupInvocationId => true,
        }
    }
    pub fn index(&self, index: usize) -> IndexedVariable {
        IndexedVariable {
            var: self.clone(),
            index,
        }
    }
    pub fn is_atomic(&self) -> bool {
        match self {
            Variable::GlobalInputArray(_, item) => item.elem().is_atomic(),
            Variable::GlobalOutputArray(_, item) => item.elem().is_atomic(),
            Variable::GlobalScalar(_, elem) => elem.is_atomic(),
            Variable::LocalMut { item, .. } => item.elem().is_atomic(),
            Variable::Named { item, .. } => item.elem().is_atomic(),
            Variable::LocalScalar { elem, .. } => elem.is_atomic(),
            Variable::SharedArray(_, item, _) => item.elem().is_atomic(),
            Variable::LocalArray(_, item, _) => item.elem().is_atomic(),
            _ => false,
        }
    }

    pub fn item(&self) -> Item {
        match self {
            Self::GlobalInputArray(_, e) => *e,
            Self::GlobalOutputArray(_, e) => *e,
            Self::SharedArray(_, e, _) => *e,
            Self::SharedValue(_, e) => *e,
            Self::ConstantArray(_, e, _) => *e,
            Self::LocalArray(_, e, _) => *e,
            Self::LocalMut { item, .. } => *item,
            Self::LocalConst { item, .. } => *item,
            Self::Named { item, .. } => *item,
            Self::Constant(_, item) => *item,
            Self::GlobalScalar(_, e) => Item::Scalar(*e),
            Self::Id => Item::Scalar(Elem::U32),
            Self::LocalInvocationIndex => Item::Scalar(Elem::U32),
            Self::LocalInvocationIdX => Item::Scalar(Elem::U32),
            Self::LocalInvocationIdY => Item::Scalar(Elem::U32),
            Self::LocalInvocationIdZ => Item::Scalar(Elem::U32),
            Self::LocalScalar { elem, .. } => Item::Scalar(*elem),
            Self::WorkgroupId => Item::Scalar(Elem::U32),
            Self::WorkgroupIdX => Item::Scalar(Elem::U32),
            Self::WorkgroupIdY => Item::Scalar(Elem::U32),
            Self::WorkgroupIdZ => Item::Scalar(Elem::U32),
            Self::GlobalInvocationIdX => Item::Scalar(Elem::U32),
            Self::GlobalInvocationIdY => Item::Scalar(Elem::U32),
            Self::GlobalInvocationIdZ => Item::Scalar(Elem::U32),
            Self::WorkgroupSize => Item::Scalar(Elem::U32),
            Self::WorkgroupSizeX => Item::Scalar(Elem::U32),
            Self::WorkgroupSizeY => Item::Scalar(Elem::U32),
            Self::WorkgroupSizeZ => Item::Scalar(Elem::U32),
            Self::NumWorkgroups => Item::Scalar(Elem::U32),
            Self::NumWorkgroupsX => Item::Scalar(Elem::U32),
            Self::NumWorkgroupsY => Item::Scalar(Elem::U32),
            Self::NumWorkgroupsZ => Item::Scalar(Elem::U32),
            Self::SubgroupSize => Item::Scalar(Elem::U32),
            Self::SubgroupId => Item::Scalar(Elem::U32),
            Self::SubgroupInvocationId => Item::Scalar(Elem::U32),
        }
    }
    pub fn elem(&self) -> Elem {
        *self.item().elem()
    }

    pub fn fmt_cast_to(&self, item: Item) -> String {
        // Noop cast.
        if self.item() == item {
            return format!("{self}");
        }

        let from = self.item();
        let from_elem = *from.elem();
        let to_elem = *item.elem();

        // Naga u64/i64 has weird limitations. We work around by first bitcasting to a 64-bit
        // type matching the target's signedness, then casting to the 32-bit target.
        let is_64bit = matches!(from_elem, Elem::I64 | Elem::U64);
        let is_32bit_target = matches!(to_elem, Elem::I32 | Elem::U32);
        if is_64bit && is_32bit_target {
            // Choose bitcast type based on target signedness (u32 -> u64, i32 -> i64)
            let bitcast_elem = if matches!(to_elem, Elem::U32) {
                Elem::U64
            } else {
                Elem::I64
            };

            if matches!(from, Scalar(_)) {
                // Scalar cast (possibly splatted to vector)
                let scalar_cast = format!("{to_elem}(bitcast<{bitcast_elem}>({self}))");
                if matches!(item, Scalar(_)) {
                    return scalar_cast;
                }
                return format!("{item}({scalar_cast})");
            }
            // Vector to vector cast
            let bitcast_item = from.with_elem(bitcast_elem);
            return format!("{item}(bitcast<{bitcast_item}>({self}))");
        }

        // WGSL doesn't support direct bool to f16 casts, can go through f32 first.
        if from_elem == Elem::Bool && to_elem == Elem::F16 {
            let f32_item = from.with_elem(Elem::F32);
            return format!("{item}({f32_item}({self}))");
        }

        // Default cases
        match (from, item) {
            // Scalar to scalar
            (Scalar(_), Scalar(_)) => format!("{item}({self})"),
            // Vec to scalar: pick first component
            (_, Scalar(_)) => format!("{item}({self}.x)"),
            (Scalar(_), _) if from_elem != to_elem => format!("{item}({to_elem}({self}))"),
            // Everything else (scalar to vec splat, vec to vec)
            _ => format!("{item}({self})"),
        }
    }
}

impl Item {
    pub fn elem(&self) -> &Elem {
        match self {
            Item::Vec4(e) => e,
            Item::Vec3(e) => e,
            Item::Vec2(e) => e,
            Item::Scalar(e) => e,
        }
    }

    pub fn size(&self) -> usize {
        self.elem().size() * self.vectorization_factor()
    }

    pub fn vectorization_factor(&self) -> usize {
        match self {
            Item::Vec4(_) => 4,
            Item::Vec3(_) => 3,
            Item::Vec2(_) => 2,
            Item::Scalar(_) => 1,
        }
    }

    pub fn with_elem(self, elem: Elem) -> Self {
        match self {
            Item::Vec4(_) => Item::Vec4(elem),
            Item::Vec3(_) => Item::Vec3(elem),
            Item::Vec2(_) => Item::Vec2(elem),
            Item::Scalar(_) => Item::Scalar(elem),
        }
    }

    pub fn fmt_cast_to(&self, item: Item, text: String) -> String {
        if *self != item {
            format!("{item}({text})")
        } else {
            text
        }
    }
}

impl Elem {
    pub fn size(&self) -> usize {
        match self {
            Self::F16 => core::mem::size_of::<half::f16>(),
            Self::F32 => core::mem::size_of::<f32>(),
            Self::F64 => core::mem::size_of::<f64>(),
            Self::AtomicF32 => core::mem::size_of::<f32>(),
            Self::I32 => core::mem::size_of::<i32>(),
            Self::I64 => core::mem::size_of::<i64>(),
            Self::AtomicI32 => core::mem::size_of::<i32>(),
            Self::U32 => core::mem::size_of::<u32>(),
            Self::U64 => core::mem::size_of::<u64>(),
            Self::AtomicU32 => core::mem::size_of::<u32>(),
            Self::Bool => core::mem::size_of::<bool>(),
        }
    }

    pub fn is_atomic(&self) -> bool {
        matches!(self, Self::AtomicI32 | Self::AtomicU32 | Self::AtomicF32)
    }
}

impl Display for Elem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F16 => f.write_str("f16"),
            Self::F32 => f.write_str("f32"),
            Self::F64 => f.write_str("f64"),
            Self::AtomicF32 => f.write_str("atomic<f32>"),
            Self::I32 => f.write_str("i32"),
            Self::I64 => f.write_str("i64"),
            Self::AtomicI32 => f.write_str("atomic<i32>"),
            Self::U32 => f.write_str("u32"),
            Self::U64 => f.write_str("u64"),
            Self::AtomicU32 => f.write_str("atomic<u32>"),
            Self::Bool => f.write_str("bool"),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Vec4(elem) => write!(f, "vec4<{elem}>"),
            Item::Vec3(elem) => write!(f, "vec3<{elem}>"),
            Item::Vec2(elem) => write!(f, "vec2<{elem}>"),
            Item::Scalar(elem) => write!(f, "{elem}"),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::GlobalInputArray(number, _) => {
                write!(f, "buffer_{number}_global")
            }
            Variable::LocalScalar { id: index, .. } => write!(f, "s_{index}"),
            Variable::LocalMut { id, .. } => write!(f, "l_mut_{id}"),
            Variable::LocalConst { id, .. } => write!(f, "l_{id}"),
            Variable::Named { name, .. } => f.write_str(name),
            Variable::GlobalOutputArray(number, _) => {
                write!(f, "buffer_{number}_global")
            }
            Variable::GlobalScalar(number, elem) => {
                write!(f, "info.scalars_{elem}[{number}]")
            }
            Variable::Constant(val, item) => {
                match (val, item.elem()) {
                    // naga can't seem to parse literals > i64::MAX or i64::MIN atm.
                    // Work around this by emitting instructions to construct these literals.
                    (ConstantValue::UInt(v), Elem::U64) if *v > i64::MAX as u64 => {
                        let as_i64 = *v as i64;
                        if as_i64 == i64::MIN {
                            write!(f, "bitcast<u64>(i64(-9223372036854775807) - 1)")
                        } else {
                            write!(f, "bitcast<u64>(i64({as_i64}))")
                        }
                    }
                    (ConstantValue::Int(v), Elem::I64) if *v == i64::MIN => {
                        write!(f, "(i64(-9223372036854775807) - 1)")
                    }
                    (_, Elem::U64) | (_, Elem::I64) => write!(f, "{item}({val})"),
                    // For other cases we can just write the val with its type.
                    _ => write!(f, "{item}({val})"),
                }
            }
            Variable::SharedArray(number, _, _) | Variable::SharedValue(number, _) => {
                write!(f, "shared_memory_{number}")
            }
            Variable::ConstantArray(number, _, _) => write!(f, "arrays_{number}"),
            Variable::LocalArray(number, _, _) => {
                write!(f, "a_{number}")
            }
            Variable::Id => f.write_str("id"),
            Variable::LocalInvocationIndex => f.write_str("local_idx"),
            Variable::LocalInvocationIdX => f.write_str("local_invocation_id.x"),
            Variable::LocalInvocationIdY => f.write_str("local_invocation_id.y"),
            Variable::LocalInvocationIdZ => f.write_str("local_invocation_id.z"),
            Variable::WorkgroupId => f.write_str("workgroup_id_no_axis"),
            Variable::WorkgroupIdX => f.write_str("workgroup_id.x"),
            Variable::WorkgroupIdY => f.write_str("workgroup_id.y"),
            Variable::WorkgroupIdZ => f.write_str("workgroup_id.z"),
            Variable::GlobalInvocationIdX => f.write_str("global_id.x"),
            Variable::GlobalInvocationIdY => f.write_str("global_id.y"),
            Variable::GlobalInvocationIdZ => f.write_str("global_id.z"),
            Variable::WorkgroupSizeX => f.write_str("WORKGROUP_SIZE_X"),
            Variable::WorkgroupSizeY => f.write_str("WORKGROUP_SIZE_Y"),
            Variable::WorkgroupSizeZ => f.write_str("WORKGROUP_SIZE_Z"),
            Variable::NumWorkgroupsX => f.write_str("num_workgroups.x"),
            Variable::NumWorkgroupsY => f.write_str("num_workgroups.y"),
            Variable::NumWorkgroupsZ => f.write_str("num_workgroups.z"),
            Variable::WorkgroupSize => f.write_str("workgroup_size_no_axis"),
            Variable::NumWorkgroups => f.write_str("num_workgroups_no_axis"),
            Variable::SubgroupSize => f.write_str("subgroup_size"),
            Variable::SubgroupId => f.write_str("subgroup_id"),
            Variable::SubgroupInvocationId => f.write_str("subgroup_invocation_id"),
        }
    }
}

impl Display for IndexedVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let var = &self.var;
        let item = var.item();
        let index = self.index;

        match &self.var {
            Variable::GlobalScalar(_, _) => write!(f, "{var}"),
            var if matches!(item, Item::Scalar(_)) => write!(f, "{var}"),
            var => write!(f, "{var}[{index}]"),
        }
    }
}

impl Variable {
    pub fn fmt_left(&self) -> String {
        match self {
            Variable::LocalConst { .. } => {
                format!("let {self}")
            }
            var => format!("{var}"),
        }
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Variable::LocalConst { .. })
    }
}

impl IndexedVariable {
    pub fn fmt_left(&self) -> String {
        let item = self.var.item();
        match &self.var {
            Variable::GlobalScalar(_, _) => self.var.fmt_left(),
            var if matches!(item, Item::Scalar(_)) => var.fmt_left(),
            _ => format!("{self}"),
        }
    }

    pub fn fmt_cast(&self, item: Item) -> String {
        if self.var.item() != item {
            format!("{item}({self})")
        } else {
            format!("{self}")
        }
    }
}

 }
 pub use base::*;
 #[derive(Debug,Clone)]pub struct Subgroup;
 impl std::fmt::Display for Subgroup{fn fmt(&self,_f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{panic!("subgroup formatter not exercised")}}
 mod extension {
use super::base::{Elem, Item, Variable};
use std::fmt::Display;

/// Not all functions are native to WGSL, so this struct allows to support more functions.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Extension {
    PowfScalar(Item),
    PowfPrimitive(Elem),
    Powf(Item),
    #[cfg(target_os = "macos")]
    SafeTanh(Item),
    #[cfg(target_os = "macos")]
    SafeTanhPrimitive(Elem),
    IsNanPrimitive(Elem),
    IsNan(Item, Item),
    IsInfPrimitive(Elem),
    IsInf(Item, Item),
}

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Extension::PowfPrimitive(elem) => format_powf_primitive(f, elem),
            Extension::PowfScalar(item) => construct_vector(
                f,
                POWF_SCALAR,
                POWF_PRIMITIVE,
                &[
                    VectorIdent {
                        name: "lhs",
                        item: *item,
                    },
                    VectorIdent {
                        name: "rhs",
                        item: Item::Scalar(*item.elem()),
                    },
                ],
                *item,
            ),
            Extension::Powf(item) => construct_vector(
                f,
                POWF,
                POWF_PRIMITIVE,
                &[
                    VectorIdent {
                        name: "lhs",
                        item: *item,
                    },
                    VectorIdent {
                        name: "rhs",
                        item: *item,
                    },
                ],
                *item,
            ),
            #[cfg(target_os = "macos")]
            Extension::SafeTanhPrimitive(elem) => format_safe_tanh_primitive(f, elem),
            #[cfg(target_os = "macos")]
            Extension::SafeTanh(item) => construct_vector(
                f,
                SAFE_TANH,
                SAFE_TANH_PRIMITIVE,
                &[VectorIdent {
                    name: "x",
                    item: *item,
                }],
                *item,
            ),
            Extension::IsNanPrimitive(elem) => format_is_nan_primitive(f, elem),
            Extension::IsNan(in_item, out_item) => construct_vector(
                f,
                IS_NAN,
                IS_NAN_PRIMITIVE,
                &[VectorIdent {
                    name: "x",
                    item: *in_item,
                }],
                *out_item,
            ),
            Extension::IsInfPrimitive(elem) => format_is_inf_primitive(f, elem),
            Extension::IsInf(in_item, out_item) => construct_vector(
                f,
                IS_INF,
                IS_INF_PRIMITIVE,
                &[VectorIdent {
                    name: "x",
                    item: *in_item,
                }],
                *out_item,
            ),
        }
    }
}

const POWF_PRIMITIVE: &str = "powf_primitive";
const POWF: &str = "powf";
const POWF_SCALAR: &str = "powf_scalar";

#[cfg(target_os = "macos")]
const SAFE_TANH_PRIMITIVE: &str = "safe_tanh_primitive";
#[cfg(target_os = "macos")]
const SAFE_TANH: &str = "safe_tanh";

const IS_NAN_PRIMITIVE: &str = "is_nan_primitive";
const IS_NAN: &str = "is_nan";

const IS_INF_PRIMITIVE: &str = "is_inf_primitive";
const IS_INF: &str = "is_inf";

pub fn powf_extension(rhs: &Variable, out: &Variable) -> Extension {
    if should_use_scalar_powf(rhs) {
        Extension::PowfScalar(out.item())
    } else {
        Extension::Powf(out.item())
    }
}

pub fn call_powf(
    f: &mut core::fmt::Formatter,
    lhs: &Variable,
    rhs: &Variable,
    out: &Variable,
) -> core::fmt::Result {
    let (lhs, rhs, base_name) = if should_use_scalar_powf(rhs) {
        let rhs = rhs.fmt_cast_to(Item::Scalar(lhs.elem()));
        let lhs = lhs.to_string();
        (lhs, rhs, POWF_SCALAR)
    } else {
        // When vectorized, we make sure the function inputs shared the same vectorization factor as
        // the output.
        let rhs = rhs.fmt_cast_to(out.item());
        let lhs = lhs.fmt_cast_to(out.item());
        (lhs, rhs, POWF)
    };
    let function_name = construct_vectorized_name(base_name, out.item());

    let out = out.fmt_left();
    write!(f, "{out} = {function_name}({lhs}, {rhs});")
}

#[cfg(target_os = "macos")]
pub fn call_safe_tanh(
    f: &mut core::fmt::Formatter,
    input: &Variable,
    out: &Variable,
) -> core::fmt::Result {
    let function_name = construct_vectorized_name(SAFE_TANH, out.item());
    let out = out.fmt_left();
    write!(f, "{out} = {function_name}({input});")
}

fn should_use_scalar_powf(rhs: &Variable) -> bool {
    rhs.is_always_scalar() || rhs.item().vectorization_factor() == 1
}

pub fn call_is_nan(
    f: &mut core::fmt::Formatter,
    input: &Variable,
    out: &Variable,
) -> core::fmt::Result {
    let function_name = construct_vectorized_name(IS_NAN, input.item());
    let out = out.fmt_left();
    write!(f, "{out} = {function_name}({input});")
}

pub fn call_is_inf(
    f: &mut core::fmt::Formatter,
    input: &Variable,
    out: &Variable,
) -> core::fmt::Result {
    let function_name = construct_vectorized_name(IS_INF, input.item());
    let out = out.fmt_left();
    write!(f, "{out} = {function_name}({input});")
}

fn construct_vectorized_name(base_name: &str, item: Item) -> String {
    let vec_factor = item.vectorization_factor();
    let elem = item.elem();
    format!("{base_name}_{vec_factor}_{elem}")
}

fn construct_primitive_name(base_name: &str, elem: Elem) -> String {
    format!("{base_name}_{elem}")
}

struct VectorIdent {
    name: &'static str,
    item: Item,
}

fn construct_vector(
    f: &mut core::fmt::Formatter<'_>,
    base_name: &str,
    primitive_name: &str,
    inputs: &[VectorIdent],
    output: Item,
) -> core::fmt::Result {
    let in_item = inputs[0].item;
    let vec_factor = output.vectorization_factor();
    let function_name = construct_vectorized_name(base_name, in_item);
    let primitive_name = construct_primitive_name(primitive_name, *in_item.elem());
    write!(f, "fn {function_name}(")?;
    for VectorIdent { name, item } in inputs {
        write!(f, "{name}: {item}, ")?;
    }
    writeln!(f, ") -> {output}{{")?;

    let indent = "    ";

    if let Item::Scalar(_) = output {
        write!(f, "{indent}return {primitive_name}(")?;
        for VectorIdent { name, item: _ } in inputs {
            write!(f, "{name}, ")?;
        }
        write!(f, ");\n}}\n")?;
        return Ok(());
    }

    writeln!(f, "{indent}return vec{vec_factor}(")?;

    for i in 0..vec_factor {
        write!(f, "{indent}{indent}{primitive_name}(")?;
        for VectorIdent { name, item } in inputs {
            match item {
                Item::Scalar(_) => {
                    write!(f, "{name}, ")?;
                }
                _ => {
                    write!(f, "{name}[{i}], ")?;
                }
            }
        }
        writeln!(f, "),")?;
    }

    write!(f, "{indent});\n}}\n")?;

    Ok(())
}

fn format_powf_primitive(
    f: &mut std::fmt::Formatter<'_>,
    elem: &Elem,
) -> Result<(), std::fmt::Error> {
    let function_name = construct_primitive_name(POWF_PRIMITIVE, *elem);
    write!(
        f,
        "
fn {function_name}(lhs: {elem}, rhs: {elem}) -> {elem} {{
    let modulo = rhs % 2.0;
    if rhs == 0.0 {{
        return 1.0;
    }}
    if (modulo == 0.0) {{
        // Even number
        return pow(abs(lhs), rhs);
    }} else if (modulo == 1.0 && lhs < 0.0) {{
        // Odd number
        return -1.0 * pow(-1.0 * lhs, rhs);
    }} else {{
        // Float number
        return pow(lhs, rhs);
    }}
}}
"
    )?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn format_safe_tanh_primitive(
    f: &mut std::fmt::Formatter<'_>,
    elem: &Elem,
) -> Result<(), std::fmt::Error> {
    let function_name = construct_primitive_name(SAFE_TANH_PRIMITIVE, *elem);
    write!(
        f,
        "
/// Metal has a weird numerical behaviour with tanh for inputs over 43.0
fn {function_name}(x: {elem}) -> {elem} {{
    if x > 43.0 {{
        return 1.0;
    }} else {{
        return tanh(x);
    }}
}}
"
    )?;
    Ok(())
}

/// Returns (`inf_bits`, `abs_mask`, unsigned type) based on the floating point data type.
fn select_inf_bits_abs_mask_uint(in_elem: &Elem) -> (&'static str, &'static str, &'static str) {
    match in_elem {
        Elem::F16 => ("0x7c00", "0x7fff", "u16"),
        Elem::F32 => ("0x7f800000u", "0x7fffffffu", "u32"),
        Elem::F64 => ("0x7fffffffffffffff", "0x7ff0000000000000", "u64"),
        _ => unreachable!(),
    }
}

fn format_is_nan_primitive(
    f: &mut std::fmt::Formatter<'_>,
    in_elem: &Elem,
) -> Result<(), std::fmt::Error> {
    let function_name = construct_primitive_name(IS_NAN_PRIMITIVE, *in_elem);
    // Per NaN definition in IEEE 754:
    //   - sign = either 0 or 1.
    //   - biased exponent = all 1 bits.
    //   - fraction = anything except all 0 bits (since all 0 bits represents infinity).
    // https://en.wikipedia.org/wiki/IEEE_754-1985#Representation_of_non-numbers

    // WGSL doesn't support u16, so for f16 we convert to f32 first (NaN/Inf are preserved)
    if matches!(in_elem, Elem::F16) {
        write!(
            f,
            "
fn {function_name}(x: f16) -> bool {{
    let bits = bitcast<u32>(f32(x));
    let abs_bits = bits & 0x7fffffffu;
    return abs_bits > 0x7f800000u;
}}
"
        )?;
    } else {
        let (inf_bits, abs_mask, uint) = select_inf_bits_abs_mask_uint(in_elem);
        write!(
            f,
            "
fn {function_name}(x: {in_elem}) -> bool {{
    let bits = bitcast<{uint}>(x);
    let abs_bits = bits & {abs_mask};
    return abs_bits > {inf_bits};
}}
"
        )?;
    }
    Ok(())
}

fn format_is_inf_primitive(
    f: &mut std::fmt::Formatter<'_>,
    in_elem: &Elem,
) -> Result<(), std::fmt::Error> {
    let function_name = construct_primitive_name(IS_INF_PRIMITIVE, *in_elem);
    // Same trick as NaN detection following IEEE 754, but check for all 0 bits equality

    // WGSL doesn't support u16, so for f16 we convert to f32 first (NaN/Inf are preserved)
    if matches!(in_elem, Elem::F16) {
        write!(
            f,
            "
fn {function_name}(x: f16) -> bool {{
    let bits = bitcast<u32>(f32(x));
    let abs_bits = bits & 0x7fffffffu;
    return abs_bits == 0x7f800000u;
}}
"
        )?;
    } else {
        let (inf_bits, abs_mask, uint) = select_inf_bits_abs_mask_uint(in_elem);
        write!(
            f,
            "
fn {function_name}(x: {in_elem}) -> bool {{
    let bits = bitcast<{uint}>(x);
    let abs_bits = bits & {abs_mask};
    return abs_bits == {inf_bits};
}}
"
        )?;
    }
    Ok(())
}

#[cfg(test)]mod audit_masks {
 use super::*;
 fn numeric(s:&str)->u64{u64::from_str_radix(s.trim_start_matches("0x").trim_end_matches('u'),16).unwrap()}
 fn classify(elem:Elem,bits:u64)->(bool,bool){let(inf,mask,_)=select_inf_bits_abs_mask_uint(&elem);let abs=bits&numeric(mask);(abs>numeric(inf),abs==numeric(inf))}
 #[test]fn f32_classification_control(){assert_eq!(classify(Elem::F32,f32::INFINITY.to_bits() as u64),(false,true));assert_eq!(classify(Elem::F32,f32::NAN.to_bits() as u64),(true,false));}
 #[test]fn f64_nan_and_infinity_are_lost_by_swapped_constants(){assert_eq!(classify(Elem::F64,f64::INFINITY.to_bits()),(false,false));assert_eq!(classify(Elem::F64,f64::NAN.to_bits()),(false,false));let nan=Extension::IsNanPrimitive(Elem::F64).to_string();let inf=Extension::IsInfPrimitive(Elem::F64).to_string();assert!(nan.contains("bits & 0x7ff0000000000000"));assert!(nan.contains("> 0x7fffffffffffffff"));assert!(inf.contains("== 0x7fffffffffffffff"));}
}

 }
 pub use extension::*;
 mod instructions {
use super::{
    Elem, Subgroup,
    base::{Item, Variable},
};
use std::fmt::Display;

/// All instructions that can be used in a WGSL compute shader.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants might not be used with different flags
pub enum Instruction {
    DeclareVariable {
        var: Variable,
    },
    Max {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Min {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Add {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Fma {
        a: Variable,
        b: Variable,
        c: Variable,
        out: Variable,
    },
    If {
        cond: Variable,
        instructions: Vec<Instruction>,
    },
    IfElse {
        cond: Variable,
        instructions_if: Vec<Instruction>,
        instructions_else: Vec<Instruction>,
    },
    Select {
        cond: Variable,
        then: Variable,
        or_else: Variable,
        out: Variable,
    },
    Switch {
        value: Variable,
        instructions_default: Vec<Instruction>,
        cases: Vec<(Variable, Vec<Instruction>)>,
    },
    Return,
    Break,
    Unreachable,
    WorkgroupBarrier,
    StorageBarrier,
    // Index handles casting to correct local variable.
    Index {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    // Index assign handles casting to correct output variable.
    IndexAssign {
        index: Variable,
        rhs: Variable,
        out: Variable,
    },
    // Index handles casting to correct local variable.
    Assign {
        input: Variable,
        out: Variable,
    },
    Modulo {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Sub {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Mul {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Div {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Abs {
        input: Variable,
        out: Variable,
    },
    Exp {
        input: Variable,
        out: Variable,
    },
    Log {
        input: Variable,
        out: Variable,
    },
    Log1p {
        input: Variable,
        out: Variable,
    },
    Cos {
        input: Variable,
        out: Variable,
    },
    Sin {
        input: Variable,
        out: Variable,
    },
    Tan {
        input: Variable,
        out: Variable,
    },
    Tanh {
        input: Variable,
        out: Variable,
    },
    Sinh {
        input: Variable,
        out: Variable,
    },
    Cosh {
        input: Variable,
        out: Variable,
    },
    ArcCos {
        input: Variable,
        out: Variable,
    },
    ArcSin {
        input: Variable,
        out: Variable,
    },
    ArcTan {
        input: Variable,
        out: Variable,
    },
    ArcSinh {
        input: Variable,
        out: Variable,
    },
    ArcCosh {
        input: Variable,
        out: Variable,
    },
    ArcTanh {
        input: Variable,
        out: Variable,
    },
    Degrees {
        input: Variable,
        out: Variable,
    },
    Radians {
        input: Variable,
        out: Variable,
    },
    ArcTan2 {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Powf {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Sqrt {
        input: Variable,
        out: Variable,
    },
    InverseSqrt {
        input: Variable,
        out: Variable,
    },
    Recip {
        input: Variable,
        out: Variable,
    },
    Equal {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Lower {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Clamp {
        input: Variable,
        min_value: Variable,
        max_value: Variable,
        out: Variable,
    },
    Greater {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    LowerEqual {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    GreaterEqual {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    NotEqual {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Length {
        var: Variable,
        out: Variable,
    },
    Metadata {
        info_offset: Variable,
        out: Variable,
    },
    ExtendedMeta {
        info_offset: Variable,
        dim: Variable,
        out: Variable,
    },
    RangeLoop {
        i: Variable,
        start: Variable,
        end: Variable,
        step: Option<Variable>,
        inclusive: bool,
        instructions: Vec<Instruction>,
    },
    And {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Or {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Not {
        input: Variable,
        out: Variable,
    },
    Loop {
        instructions: Vec<Instruction>,
    },
    BitwiseOr {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    BitwiseAnd {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    BitwiseXor {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    CountBits {
        input: Variable,
        out: Variable,
    },
    ReverseBits {
        input: Variable,
        out: Variable,
    },
    ShiftLeft {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    ShiftRight {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    BitwiseNot {
        input: Variable,
        out: Variable,
    },
    LeadingZeros {
        input: Variable,
        out: Variable,
    },
    TrailingZeros {
        input: Variable,
        out: Variable,
    },
    FindFirstSet {
        input: Variable,
        out: Variable,
    },
    Round {
        input: Variable,
        out: Variable,
    },
    Floor {
        input: Variable,
        out: Variable,
    },
    Ceil {
        input: Variable,
        out: Variable,
    },
    Trunc {
        input: Variable,
        out: Variable,
    },
    Remainder {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Slice {
        input: Variable,
        start: Variable,
        end: Variable,
        out: Variable,
    },
    CheckedSlice {
        input: Variable,
        start: Variable,
        end: Variable,
        out: Variable,
        len: Variable, // The length of the input.
    },
    Bitcast {
        input: Variable,
        out: Variable,
    },
    AtomicLoad {
        input: Variable,
        out: Variable,
    },
    AtomicStore {
        input: Variable,
        out: Variable,
    },
    AtomicSwap {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicCompareExchangeWeak {
        lhs: Variable,
        cmp: Variable,
        value: Variable,
        out: Variable,
    },
    AtomicAdd {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicSub {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicMax {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicMin {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicAnd {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicOr {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    AtomicXor {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    Subgroup(Subgroup),
    Negate {
        input: Variable,
        out: Variable,
    },
    Magnitude {
        input: Variable,
        out: Variable,
    },
    Normalize {
        input: Variable,
        out: Variable,
    },
    Dot {
        lhs: Variable,
        rhs: Variable,
        out: Variable,
    },
    VectorSum {
        input: Variable,
        out: Variable,
    },
    IsNan {
        input: Variable,
        out: Variable,
    },
    IsInf {
        input: Variable,
        out: Variable,
    },
    VecInit {
        inputs: Vec<Variable>,
        out: Variable,
    },
    Copy {
        input: Variable,
        in_index: Variable,
        out: Variable,
        out_index: Variable,
    },
    CopyBulk {
        input: Variable,
        in_index: Variable,
        out: Variable,
        out_index: Variable,
        len: u32,
    },
    Comment {
        content: String,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::DeclareVariable { var } => {
                let item = var.item();
                writeln!(f, "var {var}: {item};")
            }
            Instruction::Add { lhs, rhs, out } => {
                if out.is_atomic() {
                    assert_eq!(lhs, out, "Can't use regular addition on atomic");
                    writeln!(f, "atomicAdd({out}, {rhs});")
                } else {
                    let lhs = lhs.fmt_cast_to(out.item());
                    let rhs = rhs.fmt_cast_to(out.item());
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {lhs} + {rhs};")
                }
            }
            Instruction::Slice {
                input,
                start,
                end,
                out,
            } => {
                writeln!(f, "let {out}_offset = {start};")?;
                writeln!(f, "let {out}_length = {end} - {start};")?;
                writeln!(f, "let {out}_ptr = &{input};")
            }
            Instruction::CheckedSlice {
                input,
                start,
                end,
                out,
                len,
            } => {
                writeln!(f, "let {out}_offset = {start};")?;
                writeln!(f, "let {out}_length = min({len}, {end}) - {start};")?;
                writeln!(f, "let {out}_ptr = &{input};")
            }
            Instruction::Fma { a, b, c, out } => {
                let a = a.fmt_cast_to(out.item());
                let b = b.fmt_cast_to(out.item());
                let c = c.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = fma({a}, {b}, {c});")
            }
            Instruction::Min { lhs, rhs, out } => {
                if out.is_atomic() {
                    assert_eq!(lhs, out, "Can't use regular min on atomic");
                    writeln!(f, "atomicMin({out}, {rhs});")
                } else {
                    let lhs = lhs.fmt_cast_to(out.item());
                    let rhs = rhs.fmt_cast_to(out.item());
                    let out = out.fmt_left();
                    writeln!(f, "{out} = min({lhs}, {rhs});")
                }
            }
            Instruction::Max { lhs, rhs, out } => {
                if out.is_atomic() {
                    assert_eq!(lhs, out, "Can't use regular max on atomic");
                    writeln!(f, "atomicMax({out}, {rhs});")
                } else {
                    let lhs = lhs.fmt_cast_to(out.item());
                    let rhs = rhs.fmt_cast_to(out.item());
                    let out = out.fmt_left();
                    writeln!(f, "{out} = max({lhs}, {rhs});")
                }
            }
            Instruction::And { lhs, rhs, out } => {
                let vector_size = out.item().vectorization_factor();
                if out.is_atomic() {
                    assert_eq!(lhs, out, "Can't use regular and on atomic");
                    writeln!(f, "atomicAnd({out}, {rhs});")
                } else if vector_size > 1 {
                    let item = out.item();
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {item}(")?;
                    for i in 0..vector_size {
                        let lhs_i = lhs.index(i);
                        let rhs_i = rhs.index(i);
                        writeln!(f, "{lhs_i} && {rhs_i},")?;
                    }
                    writeln!(f, ");")
                } else {
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {lhs} && {rhs};")
                }
            }
            Instruction::Or { lhs, rhs, out } => {
                let vector_size = out.item().vectorization_factor();
                if out.is_atomic() {
                    assert_eq!(lhs, out, "Can't use regular or on atomic");
                    writeln!(f, "atomicOr({out}, {rhs});")
                } else if vector_size > 1 {
                    let item = out.item();
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {item}(")?;
                    for i in 0..vector_size {
                        let lhs_i = lhs.index(i);
                        let rhs_i = rhs.index(i);
                        writeln!(f, "{lhs_i} || {rhs_i},")?;
                    }
                    writeln!(f, ");")
                } else {
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {lhs} || {rhs};")
                }
            }
            Instruction::Not { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = !{input};")
            }
            Instruction::Index { lhs, rhs, out } => index(f, lhs, rhs, out, None, None),
            Instruction::IndexAssign {
                index: lhs,
                rhs,
                out,
            } => index_assign(f, lhs, rhs, out, None),
            Instruction::Copy {
                input,
                in_index,
                out,
                out_index,
            } => {
                let rhs = format!("{input}[{in_index}]");
                let lhs = format!("{out}[{out_index}]");
                writeln!(f, "{lhs} = {rhs};")
            }
            Instruction::CopyBulk {
                input,
                in_index,
                out,
                out_index,
                len,
            } => {
                for i in 0..*len {
                    let rhs = format!("{input}[{in_index} + {i}]");
                    let lhs = format!("{out}[{out_index} + {i}]");
                    writeln!(f, "{lhs} = {rhs};")?;
                }
                Ok(())
            }
            Instruction::Modulo { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} % {rhs};")
            }
            Instruction::Remainder { lhs, rhs, out } => {
                let f_type = out.item().with_elem(Elem::F32);
                let ty = out.item();
                let lhs_f = lhs.fmt_cast_to(f_type);
                let rhs_f = rhs.fmt_cast_to(f_type);
                let lhs = lhs.fmt_cast_to(ty);
                let rhs = rhs.fmt_cast_to(ty);
                let out = out.fmt_left();
                let floor = f_type.fmt_cast_to(ty, format!("floor({lhs_f} / {rhs_f})"));
                writeln!(f, "{out} = {lhs} - {rhs} * {floor};")
            }
            Instruction::Sub { lhs, rhs, out } => {
                if out.is_atomic() {
                    assert_eq!(lhs, out, "Can't use regular sub on atomic");
                    writeln!(f, "atomicSub({out}, {rhs});")
                } else {
                    let lhs = lhs.fmt_cast_to(out.item());
                    let rhs = rhs.fmt_cast_to(out.item());
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {lhs} - {rhs};")
                }
            }
            Instruction::Mul { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} * {rhs};")
            }
            Instruction::Div { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} / {rhs};")
            }
            Instruction::Abs { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = abs({input});")
            }
            Instruction::Exp { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = exp({input});")
            }
            Instruction::Log { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = log({input});")
            }
            Instruction::Clamp {
                input,
                min_value,
                max_value,
                out,
            } => {
                let min = min_value.fmt_cast_to(out.item());
                let max = max_value.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = clamp({input}, {min}, {max});")
            }
            Instruction::Powf { lhs, rhs, out } => super::call_powf(f, lhs, rhs, out),
            Instruction::IsNan { input, out } => super::call_is_nan(f, input, out),
            Instruction::IsInf { input, out } => super::call_is_inf(f, input, out),
            Instruction::Sqrt { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = sqrt({input});")
            }
            Instruction::InverseSqrt { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = inverseSqrt({input});")
            }
            Instruction::Log1p { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = log({input} + 1.0);")
            }
            Instruction::Cos { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = cos({input});")
            }
            Instruction::Sin { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = sin({input});")
            }
            Instruction::Tan { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = tan({input});")
            }
            Instruction::Tanh { input, out } => {
                #[cfg(target_os = "macos")]
                let result = super::call_safe_tanh(f, input, out);
                #[cfg(not(target_os = "macos"))]
                let result = {
                    let out = out.fmt_left();
                    writeln!(f, "{out} = tanh({input});")
                };

                result
            }
            Instruction::Sinh { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = sinh({input});")
            }
            Instruction::Cosh { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = cosh({input});")
            }
            Instruction::ArcCos { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = acos({input});")
            }
            Instruction::ArcSin { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = asin({input});")
            }
            Instruction::ArcTan { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = atan({input});")
            }
            Instruction::ArcSinh { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = asinh({input});")
            }
            Instruction::ArcCosh { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = acosh({input});")
            }
            Instruction::ArcTanh { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = atanh({input});")
            }
            Instruction::Degrees { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = degrees({input});")
            }
            Instruction::Radians { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = radians({input});")
            }
            Instruction::ArcTan2 { lhs, rhs, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = atan2({lhs}, {rhs});")
            }
            Instruction::Recip { input, out } => {
                let item = input.item();
                let out = out.fmt_left();
                write!(f, "{out} = {item}(1.0) / {input};")
            }
            Instruction::Equal { lhs, rhs, out } => comparison(lhs, rhs, out, "==", f),
            Instruction::Lower { lhs, rhs, out } => comparison(lhs, rhs, out, "<", f),
            Instruction::Greater { lhs, rhs, out } => comparison(lhs, rhs, out, ">", f),
            Instruction::LowerEqual { lhs, rhs, out } => comparison(lhs, rhs, out, "<=", f),
            Instruction::GreaterEqual { lhs, rhs, out } => comparison(lhs, rhs, out, ">=", f),
            Instruction::NotEqual { lhs, rhs, out } => comparison(lhs, rhs, out, "!=", f),
            Instruction::Assign { input, out } => {
                let vec_left = out.item().vectorization_factor();
                let vec_right = input.item().vectorization_factor();

                if out.elem().is_atomic() {
                    if !input.is_atomic() {
                        writeln!(f, "let {out} = {input};")
                    } else {
                        writeln!(f, "let {out} = &{input};")
                    }
                } else if vec_left != vec_right {
                    if vec_right == 1 {
                        let input = input.fmt_cast_to(out.item());
                        let out = out.fmt_left();
                        writeln!(f, "{out} = {input};")
                    } else {
                        for i in 0..vec_right {
                            let out = out.index(i);
                            let input = input.index(i);
                            writeln!(f, "{out} = {input};")?;
                        }
                        Ok(())
                    }
                } else {
                    let input = input.fmt_cast_to(out.item());
                    let out = out.fmt_left();
                    writeln!(f, "{out} = {input};")
                }
            }
            Instruction::Metadata { info_offset, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = info.static_meta[{info_offset}];")
            }
            Instruction::ExtendedMeta {
                dim,
                info_offset,
                out,
            } => {
                let out = out.fmt_left();
                writeln!(
                    f,
                    "{out} = info.dynamic_meta[info.static_meta[{info_offset}] + {dim}];"
                )
            }
            Instruction::RangeLoop {
                i,
                start,
                end,
                step,
                inclusive,
                instructions,
            } => {
                let increment = step
                    .as_ref()
                    .map(|step| format!("{i} += {step}"))
                    .unwrap_or_else(|| format!("{i}++"));
                let cmp = if *inclusive { "<=" } else { "<" };
                let i_ty = i.item();

                write!(
                    f,
                    "
for (var {i}: {i_ty} = {start}; {i} {cmp} {end}; {increment}) {{
"
                )?;
                for instruction in instructions {
                    write!(f, "{instruction}")?;
                }

                f.write_str("}\n")
            }
            Instruction::If { cond, instructions } => {
                writeln!(f, "if {cond} {{")?;
                for i in instructions {
                    write!(f, "{i}")?;
                }
                f.write_str("}\n")
            }
            Instruction::IfElse {
                cond,
                instructions_if,
                instructions_else,
            } => {
                writeln!(f, "if {cond} {{")?;
                for i in instructions_if {
                    write!(f, "{i}")?;
                }
                f.write_str("} else {\n")?;
                for i in instructions_else {
                    write!(f, "{i}")?;
                }
                f.write_str("}\n")
            }
            Instruction::Select {
                cond,
                then,
                or_else,
                out,
            } => {
                let bool_ty = out.item().with_elem(Elem::Bool);

                let cond = cond.fmt_cast_to(bool_ty);
                let then = then.fmt_cast_to(out.item());
                let or_else = or_else.fmt_cast_to(out.item());
                let out = out.fmt_left();

                writeln!(f, "{out} = select({or_else}, {then}, {cond});")
            }
            Instruction::Switch {
                value,
                instructions_default,
                cases,
            } => {
                writeln!(f, "switch({value}) {{")?;
                for (val, block) in cases {
                    writeln!(f, "case {val}: {{")?;
                    for i in block {
                        i.fmt(f)?;
                    }
                    f.write_str("}\n")?;
                }
                f.write_str("default: {\n")?;
                for i in instructions_default {
                    i.fmt(f)?;
                }
                f.write_str("}\n}\n")
            }
            Instruction::Return => f.write_str("return;\n"),
            Instruction::Break => f.write_str("break;\n"),
            Instruction::WorkgroupBarrier => f.write_str("workgroupBarrier();\n"),
            Instruction::StorageBarrier => f.write_str("storageBarrier();\n"),
            Instruction::Length { var, out } => {
                let out = out.fmt_left();

                match var {
                    Variable::ConstantArray(_, _, length) => {
                        writeln!(f, "{out} = {length}u;")
                    }
                    Variable::LocalArray(_, _, length) => {
                        writeln!(f, "{out} = {length}u;")
                    }
                    Variable::SharedArray(_, _, length) => {
                        writeln!(f, "{out} = {length}u;")
                    }
                    _ => {
                        writeln!(f, "{out} = arrayLength({var});")
                    }
                }
            }
            Instruction::Loop { instructions } => {
                writeln!(f, "loop {{")?;
                for i in instructions {
                    write!(f, "{i}")?;
                }
                f.write_str("}\n")
            }
            Instruction::BitwiseOr { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} | {rhs};")
            }
            Instruction::BitwiseAnd { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} & {rhs};")
            }
            Instruction::BitwiseXor { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item());
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} ^ {rhs};")
            }
            Instruction::CountBits { input, out } => {
                let out_item = out.item();
                let out = out.fmt_left();
                match input.elem() == *out_item.elem() {
                    true => writeln!(f, "{out} = countOneBits({input});"),
                    false => writeln!(f, "{out} = {out_item}(countOneBits({input}));"),
                }
            }
            Instruction::ReverseBits { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = reverseBits({input});")
            }
            Instruction::ShiftLeft { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item().with_elem(Elem::U32));
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} << {rhs};")
            }
            Instruction::ShiftRight { lhs, rhs, out } => {
                let lhs = lhs.fmt_cast_to(out.item());
                let rhs = rhs.fmt_cast_to(out.item().with_elem(Elem::U32));
                let out = out.fmt_left();
                writeln!(f, "{out} = {lhs} >> {rhs};")
            }
            Instruction::BitwiseNot { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = ~{input};")
            }
            Instruction::LeadingZeros { input, out } => {
                let out_fmt = out.fmt_left();
                match input.elem() {
                    // 64-bit polyfill: split into upper/lower 32 bits
                    Elem::I64 | Elem::U64 => {
                        let u64_item = input.item().with_elem(Elem::U64);
                        let u32_item = input.item().with_elem(Elem::U32);
                        let input = input.fmt_cast_to(u64_item);
                        writeln!(
                            f,
                            "{out_fmt} = select(countLeadingZeros({u32_item}({input} >> {u32_item}(32u))), 32u + countLeadingZeros({u32_item}({input})), ({input} >> {u32_item}(32u)) == {u64_item}(0));"
                        )
                    }
                    _ => {
                        let input = input.fmt_cast_to(input.item().with_elem(Elem::U32));
                        writeln!(f, "{out_fmt} = countLeadingZeros({input});")
                    }
                }
            }
            Instruction::TrailingZeros { input, out } => {
                let out_fmt = out.fmt_left();
                match input.elem() {
                    // 64-bit polyfill: split into upper/lower 32 bits
                    Elem::I64 | Elem::U64 => {
                        let u64_item = input.item().with_elem(Elem::U64);
                        let u32_item = input.item().with_elem(Elem::U32);
                        let input = input.fmt_cast_to(u64_item);
                        writeln!(
                            f,
                            "{out_fmt} = select(countTrailingZeros({u32_item}({input})), 32u + countTrailingZeros({u32_item}({input} >> {u32_item}(32u))), {u32_item}({input}) == {u32_item}(0u));"
                        )
                    }
                    _ => {
                        let input = input.fmt_cast_to(input.item().with_elem(Elem::U32));
                        writeln!(f, "{out_fmt} = countTrailingZeros({input});")
                    }
                }
            }
            Instruction::FindFirstSet { input, out } => {
                let out_fmt = out.fmt_left();
                match input.elem() {
                    // 64-bit polyfill: split into upper/lower 32 bits
                    Elem::I64 | Elem::U64 => {
                        let u64_item = input.item().with_elem(Elem::U64);
                        let u32_item = input.item().with_elem(Elem::U32);
                        let input = input.fmt_cast_to(u64_item);
                        writeln!(
                            f,
                            "{out_fmt} = select(firstTrailingBit({u32_item}({input})) + 1, select(firstTrailingBit({u32_item}({input} >> {u32_item}(32u))) + 33, {u32_item}(0u), ({input} >> {u32_item}(32u)) == {u64_item}(0)), {u32_item}({input}) == {u32_item}(0u));"
                        )
                    }
                    _ => {
                        let input = input.fmt_cast_to(input.item().with_elem(Elem::U32));
                        writeln!(f, "{out_fmt} = firstTrailingBit({input}) + 1;")
                    }
                }
            }
            Instruction::Round { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = round({input});")
            }
            Instruction::Floor { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = floor({input});")
            }
            Instruction::Ceil { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = ceil({input});")
            }
            Instruction::Trunc { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = trunc({input});")
            }
            Instruction::Subgroup(op) => write!(f, "{op}"),
            Instruction::Bitcast { input, out } => {
                let elem = out.item();
                let out = out.fmt_left();
                writeln!(f, "{out} = bitcast<{elem}>({input});")
            }
            Instruction::AtomicLoad { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = atomicLoad({input});")
            }
            Instruction::AtomicStore { input, out } => {
                writeln!(f, "atomicStore({out},{input});")
            }
            Instruction::AtomicSwap { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicExchange({lhs}, {rhs});")
            }
            Instruction::AtomicAdd { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicAdd({lhs}, {rhs});")
            }
            Instruction::AtomicSub { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicSub({lhs}, {rhs});")
            }
            Instruction::AtomicMax { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicMax({lhs}, {rhs});")
            }
            Instruction::AtomicMin { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicMin({lhs}, {rhs});")
            }
            Instruction::AtomicAnd { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicAnd({lhs}, {rhs});")
            }
            Instruction::AtomicOr { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicOr({lhs}, {rhs});")
            }
            Instruction::AtomicXor { lhs, rhs, out } => {
                let out = out.fmt_left();
                write!(f, "{out} = atomicXor({lhs}, {rhs});")
            }
            Instruction::AtomicCompareExchangeWeak {
                lhs,
                cmp,
                value,
                out,
            } => {
                let out = out.fmt_left();
                writeln!(
                    f,
                    // For compatibility with cuda, only return old_value
                    "{out} = atomicCompareExchangeWeak({lhs}, {cmp}, {value}).old_value;"
                )
            }
            Instruction::Negate { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = -{input};")
            }
            Instruction::Magnitude { input, out } => {
                let out = out.fmt_left();
                writeln!(f, "{out} = length({input});")
            }
            Instruction::Normalize { input, out } => {
                if input.item().vectorization_factor() == 1 {
                    // We need a check for vectorization factor 1 here, for compatibility with cuda.
                    // You can almost use sign here, however that does not correctly handle the case for x == 0.0.
                    // Therefore we use normalize with vec2, as there is no way to use a NaN literal in wgsl.
                    let vec2_type = Item::Vec2(out.elem());
                    let out = out.fmt_left();
                    writeln!(f, "{out} = normalize({vec2_type}({input}, 0.0)).x;")
                } else {
                    let out = out.fmt_left();
                    writeln!(f, "{out} = normalize({input});")
                }
            }
            Instruction::Dot { lhs, rhs, out } => {
                let out = out.fmt_left();
                if lhs.item().vectorization_factor() == 1 {
                    writeln!(f, "{out} = {lhs} * {rhs};")
                } else {
                    writeln!(f, "{out} = dot({lhs}, {rhs});")
                }
            }
            Instruction::VectorSum { input, out } => {
                let vec_size = input.item().vectorization_factor();
                let out = out.fmt_left();
                if vec_size <= 1 {
                    writeln!(f, "{out} = {input};")
                } else {
                    let elems = (0..vec_size)
                        .map(|i| format!("{}", input.index(i)))
                        .collect::<Vec<_>>();
                    writeln!(f, "{out} = {};", elems.join(" + "))
                }
            }
            Instruction::VecInit { inputs, out } => {
                let item = out.item();
                let inputs = inputs.iter().map(|var| var.to_string()).collect::<Vec<_>>();
                let out = out.fmt_left();
                writeln!(f, "{out} = {item}({});", inputs.join(", "))
            }
            Instruction::Comment { content } => {
                if content.contains('\n') {
                    writeln!(f, "/* {content} */")
                } else {
                    writeln!(f, "// {content}")
                }
            }
            // WGSL as usual has no lower level intrinsics
            Instruction::Unreachable => writeln!(f, "return;"),
        }
    }
}

fn comparison(
    lhs: &Variable,
    rhs: &Variable,
    out: &Variable,
    op: &str,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let item = out.item().with_elem(lhs.elem());
    let lhs = lhs.fmt_cast_to(item);
    let rhs = rhs.fmt_cast_to(item);
    let out = out.fmt_left();
    writeln!(f, "{out} = {lhs} {op} {rhs};")
}

struct IndexOffset {
    var: Variable,
    offset: Option<Variable>,
    index: usize,
}
impl IndexOffset {
    fn new(var: &Variable, offset: &Option<Variable>, index: usize) -> Self {
        Self {
            var: var.clone(),
            offset: offset.clone(),
            index,
        }
    }
}

impl Display for IndexOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let var = self.var.index(self.index);

        match &self.offset {
            Some(offset) => {
                let offset = offset.index(self.index);
                write!(f, "{var} + {offset}")
            }
            None => write!(f, "{var}"),
        }
    }
}

fn index(
    f: &mut std::fmt::Formatter<'_>,
    lhs: &Variable,
    rhs: &Variable,
    out: &Variable,
    offset: Option<Variable>,
    len: Option<&Variable>,
) -> core::fmt::Result {
    let is_scalar = match lhs {
        Variable::LocalMut { item, .. } => item.vectorization_factor() == 1,
        Variable::LocalConst { item, .. } => item.vectorization_factor() == 1,
        Variable::Constant(..) => true,
        _ => false,
    };

    let (mut value, index) = if is_scalar {
        (format!("{lhs}"), None)
    } else {
        let value = if let Some(offset) = offset {
            format!("{lhs}[{rhs}+{offset}]")
        } else {
            format!("{lhs}[{rhs}]")
        };

        (value, Some(format!("{rhs}")))
    };

    if out.item().elem().is_atomic() {
        // Atomic values don't support casting or bound checking - we just assign the reference.
        value = format!("&{value}");
        writeln!(f, "let {out} = {value};")
    } else {
        // Check for casting
        if lhs.elem() != out.elem() {
            value = lhs.item().fmt_cast_to(out.item(), value)
        };

        // Check for bounds.
        if let Some(ind) = index
            && let Some(len) = len
        {
            // Note: This is technically not 100% allowed. According to the WebGPU specification,
            // any OOB access is a "dynamic error" which allows "many possible outcomes". In practice,
            // both wgpu and Dawn handle this by either returning dummy data or clamping the index
            // to valid bounds. This means it's harmless to use in a select.
            let out_item = out.item();
            value = format!("select({out_item}(0), {value}, {ind} < {len})");
        };

        let out = out.fmt_left();
        writeln!(f, "{out} = {value};")
    }
}

fn index_assign(
    f: &mut std::fmt::Formatter<'_>,
    lhs: &Variable,
    rhs: &Variable,
    out: &Variable,
    offset: Option<Variable>,
) -> core::fmt::Result {
    match lhs.item() {
        Item::Vec4(elem) => {
            let item = Item::Scalar(elem);
            let lhs0 = IndexOffset::new(lhs, &offset, 0);

            let rhs0 = rhs.index(0).fmt_cast(item);
            let rhs1 = rhs.index(1).fmt_cast(item);
            let rhs2 = rhs.index(2).fmt_cast(item);
            let rhs3 = rhs.index(3).fmt_cast(item);

            write!(f, "{out}[{lhs0}] = vec4({rhs0}, {rhs1}, {rhs2}, {rhs3})")
        }
        Item::Vec3(elem) => {
            let item = Item::Scalar(elem);
            let lhs0 = IndexOffset::new(lhs, &offset, 0);

            let rhs0 = rhs.index(0).fmt_cast(item);
            let rhs1 = rhs.index(1).fmt_cast(item);
            let rhs2 = rhs.index(2).fmt_cast(item);

            writeln!(f, "{out}[{lhs0}] = vec3({rhs0}, {rhs1}, {rhs2});")
        }
        Item::Vec2(elem) => {
            let item = Item::Scalar(elem);
            let lhs0 = IndexOffset::new(lhs, &offset, 0);

            let rhs0 = rhs.index(0).fmt_cast(item);
            let rhs1 = rhs.index(1).fmt_cast(item);

            writeln!(f, "{out}[{lhs0}] = vec2({rhs0}, {rhs1});")
        }
        Item::Scalar(_elem) => {
            let is_array = match out {
                Variable::GlobalInputArray(_, _)
                | Variable::GlobalOutputArray(_, _)
                | Variable::SharedArray(_, _, _)
                | Variable::LocalArray(_, _, _) => true,
                Variable::Named { is_array, .. } => *is_array,
                _ => false,
            };

            if !is_array {
                let elem_out = out.elem();
                let casting_type = match rhs.item() {
                    Item::Vec4(_) => Item::Vec4(elem_out),
                    Item::Vec3(_) => Item::Vec3(elem_out),
                    Item::Vec2(_) => Item::Vec2(elem_out),
                    Item::Scalar(_) => Item::Scalar(elem_out),
                };
                let rhs = rhs.fmt_cast_to(casting_type);
                if matches!(out.item(), Item::Scalar(_)) {
                    writeln!(f, "{out} = {rhs};")
                } else {
                    writeln!(f, "{out}[{lhs}] = {rhs};")
                }
            } else {
                let item_rhs = rhs.item();
                let item_out = out.item();
                let lhs = IndexOffset::new(lhs, &offset, 0);

                let vectorization_factor = item_out.vectorization_factor();
                if vectorization_factor > item_rhs.vectorization_factor() {
                    let casting_type = Item::Scalar(*item_out.elem());
                    write!(f, "{out}[{lhs}] = vec{vectorization_factor}(")?;
                    for i in 0..vectorization_factor {
                        f.write_str(&rhs.index(i).fmt_cast(casting_type))?;

                        if i < vectorization_factor - 1 {
                            f.write_str(",")?;
                        }
                    }
                    f.write_str(");\n")
                } else {
                    let rhs = rhs.fmt_cast_to(item_out);
                    writeln!(f, "{out}[{lhs}] = {rhs};")
                }
            }
        }
    }
}

 }
 pub use instructions::*;
 use crate::compiler::wgsl as wgsl;
fn register_extensions(instructions: &[wgsl::Instruction]) -> Vec<wgsl::Extension> {
    let mut extensions = Vec::new();

    let mut register_extension = |extension: wgsl::Extension| {
        if !extensions.contains(&extension) {
            extensions.push(extension);
        }
    };

    // Since not all instructions are native to WGSL, we need to add the custom ones.
    for instruction in instructions {
        match instruction {
            wgsl::Instruction::Powf { lhs: _, rhs, out } => {
                register_extension(wgsl::Extension::PowfPrimitive(out.elem()));
                register_extension(wgsl::powf_extension(rhs, out));
            }
            #[cfg(target_os = "macos")]
            wgsl::Instruction::Tanh { input, out: _ } => {
                register_extension(wgsl::Extension::SafeTanhPrimitive(input.elem()));
                register_extension(wgsl::Extension::SafeTanh(input.item()));
            }
            wgsl::Instruction::IsNan { input, out } => {
                register_extension(wgsl::Extension::IsNanPrimitive(input.elem()));
                register_extension(wgsl::Extension::IsNan(input.item(), out.item()));
            }
            wgsl::Instruction::IsInf { input, out } => {
                register_extension(wgsl::Extension::IsInfPrimitive(input.elem()));
                register_extension(wgsl::Extension::IsInf(input.item(), out.item()));
            }
            wgsl::Instruction::If { instructions, .. } => {
                for extension in register_extensions(instructions) {
                    register_extension(extension);
                }
            }
            wgsl::Instruction::IfElse {
                instructions_if,
                instructions_else,
                ..
            } => {
                for extension in register_extensions(instructions_if) {
                    register_extension(extension);
                }
                for extension in register_extensions(instructions_else) {
                    register_extension(extension);
                }
            }
            wgsl::Instruction::Loop { instructions } => {
                for extension in register_extensions(instructions) {
                    register_extension(extension);
                }
            }
            wgsl::Instruction::RangeLoop { instructions, .. } => {
                for extension in register_extensions(instructions) {
                    register_extension(extension);
                }
            }
            _ => {}
        }
    }

    extensions
}
 #[cfg(test)]mod audit {
 use super::*;
 fn input()->Variable{Variable::Named{name:"x".into(),item:Item::Scalar(Elem::F32),is_array:false}}
 fn output()->Variable{Variable::LocalMut{id:1,item:Item::Scalar(Elem::Bool)}}
 fn check()->Instruction{Instruction::IsNan{input:input(),out:output()}}
 #[test]fn direct_helper_registration_control(){let e=register_extensions(&[check()]);assert_eq!(e.len(),2);let calls=check().to_string();let defs=e.iter().map(ToString::to_string).collect::<String>();assert!(calls.contains("is_nan_1_f32"));assert!(defs.contains("fn is_nan_1_f32"));}
 #[test]fn if_loop_and_range_registration_control(){
  let wrappers=vec![Instruction::IfElse{cond:output(),instructions_if:vec![],instructions_else:vec![check()]},Instruction::Loop{instructions:vec![check()]},Instruction::RangeLoop{i:input(),start:input(),end:input(),step:None,inclusive:false,instructions:vec![check()]}];
  for wrapper in wrappers {assert_eq!(register_extensions(&[wrapper]).len(),2);}
 }
 #[test]fn switch_case_calls_helper_without_registering_definition(){let switch=Instruction::Switch{value:input(),instructions_default:vec![],cases:vec![(input(),vec![check()])]};assert!(switch.to_string().contains("is_nan_1_f32"));assert!(register_extensions(&[switch]).is_empty());}
 #[test]fn switch_default_calls_helper_without_registering_definition(){let switch=Instruction::Switch{value:input(),instructions_default:vec![check()],cases:vec![]};assert!(switch.to_string().contains("is_nan_1_f32"));assert!(register_extensions(&[switch]).is_empty());}
 #[test]fn vec4_index_assignment_omits_statement_terminator(){
  let instruction=Instruction::IndexAssign{index:Variable::LocalMut{id:0,item:Item::Vec4(Elem::U32)},rhs:Variable::LocalMut{id:1,item:Item::Vec4(Elem::F32)},out:Variable::GlobalOutputArray(0,Item::Vec4(Elem::F32))};
  let text=instruction.to_string();assert!(text.contains("= vec4("));assert!(!text.trim_end().ends_with(';'));
 }
 #[test]fn vec3_index_assignment_terminator_control(){
  let instruction=Instruction::IndexAssign{index:Variable::LocalMut{id:0,item:Item::Vec3(Elem::U32)},rhs:Variable::LocalMut{id:1,item:Item::Vec3(Elem::F32)},out:Variable::GlobalOutputArray(0,Item::Vec3(Elem::F32))};assert!(instruction.to_string().trim_end().ends_with(';'));
 }
 }
}}
mod storage_predicate {
 const TRUE:u32=1;
 #[derive(Clone,Copy)]struct Storage16{uniform_and_storage_buffer16_bit_access:u32}
 #[derive(Clone,Copy)]struct Storage8{uniform_and_storage_buffer8_bit_access:u32}
 struct ExtendedFeatures{buf_16:Option<Storage16>,buf_8:Option<Storage8>}
 fn predicates(ext_feat:&ExtendedFeatures)->(bool,bool){
let storage16 = ext_feat
        .buf_16
        .is_some_and(|it| it.uniform_and_storage_buffer16_bit_access == TRUE);
    let storage8 = ext_feat
        .buf_16
        .is_some_and(|it| it.uniform_and_storage_buffer16_bit_access == TRUE);

    
 (storage16,storage8)
 }
 #[test]fn sixteen_only_falsely_admits_eight(){assert_eq!(predicates(&ExtendedFeatures{buf_16:Some(Storage16{uniform_and_storage_buffer16_bit_access:1}),buf_8:None}),(true,true));}
 #[test]fn eight_only_is_falsely_excluded(){assert_eq!(predicates(&ExtendedFeatures{buf_16:None,buf_8:Some(Storage8{uniform_and_storage_buffer8_bit_access:1})}),(false,false));}
 #[test]fn both_absent_control(){assert_eq!(predicates(&ExtendedFeatures{buf_16:None,buf_8:None}),(false,false));}
 #[test]fn both_present_control(){assert_eq!(predicates(&ExtendedFeatures{buf_16:Some(Storage16{uniform_and_storage_buffer16_bit_access:1}),buf_8:Some(Storage8{uniform_and_storage_buffer8_bit_access:1})}),(true,true));}
}
