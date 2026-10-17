#![allow(dead_code)]
trait Comptime:Sized{fn comptime(self)->Self{self}}impl Comptime for u32{}
trait Size{fn value()->usize;}struct Width<const V:usize>;impl<const V:usize>Size for Width<V>{fn value()->usize{V}}
struct E2;impl E2{fn cast_from(v:i32)->i32{v}}
struct Layout{num_rows:u32,num_cols:u32}
struct UnitTile{data:Vec<i32>,layout:Layout}
struct Vector(Vec<i32>);impl Vector{fn extract(&self,i:usize)->i32{self.0[i]}}
struct StridedTile{data:Vec<i32>,input_cols:usize,vector_size:usize}
impl StridedTile{fn get_vector(&self,row:u32,col:u32)->Vector{let start=row as usize*self.input_cols+col as usize*self.vector_size;Vector(self.data[start..start+self.vector_size].to_vec())}}
fn load<N:Size>(strided_tile:&StridedTile,unit_tile:&mut UnitTile){
    let vector_size = N::value().comptime() as u32;
    assert!(unit_tile.layout.num_cols.is_multiple_of(vector_size));

    let input_num_rows = unit_tile.layout.num_cols.comptime();
    let input_num_cols = unit_tile.layout.num_rows.comptime();
    let vector_iterations = input_num_cols / vector_size;

    for input_row in 0..input_num_rows {
        for input_col_vector in 0..vector_iterations {
            let vector_read = strided_tile.get_vector(input_row, input_col_vector);

            
            for i in 0..vector_size {
                unit_tile.data[((input_col_vector + i) * input_num_rows + input_row) as usize] =
                    E2::cast_from(vector_read.extract(i as usize));
            }
        }
    }
}

fn run<const V:usize>(rows:u32,cols:u32)->Vec<i32>{let n=(rows*cols)as usize;let input=StridedTile{data:(0..n as i32).collect(),input_cols:rows as usize,vector_size:V};let mut output=UnitTile{data:vec![-999;n],layout:Layout{num_rows:rows,num_cols:cols}};load::<Width<V>>(&input,&mut output);output.data}
fn transpose(rows:u32,cols:u32)->Vec<i32>{(0..rows).flat_map(|r|(0..cols).map(move|c|(c*rows+r)as i32)).collect()}
#[test]fn vector2_transpose_overwrites_and_leaves_holes(){let got=run::<2>(4,2);assert_eq!(got,vec![0,4,2,6,3,7,-999,-999]);assert_ne!(got,transpose(4,2));}
#[test]fn vector1_control_is_correct(){assert_eq!(run::<1>(4,2),transpose(4,2));}
#[test]fn vector2_single_input_vector_control(){assert_eq!(run::<2>(2,2),transpose(2,2));}
#[test]fn valid_transposed_input_contiguous_axis_is_rejected_by_wrong_axis_assert(){assert_eq!(2u32%2,0);assert!(std::panic::catch_unwind(||run::<2>(2,3)).is_err());}
