pub type Index = u32;
pub type InputIndex = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Op {
    And = 0,
    Or = 1,
    Xor = 2
}

pub trait GateSoup {
    fn eval(&self,constraints:&Vec<(Index,bool)>)->Vec<bool>;
    fn dump(&self,path:&str)->Result<(),std::io::Error>;
    fn input(&self,i:InputIndex)->Index;
    fn new_input(&self)->Index;
    fn num_inputs(&self)->usize;
    fn binop(&self,op:Op,a:Index,b:Index)->Index;
    fn and(&self,a:Index,b:Index)->Index;
    fn or(&self,a:Index,b:Index)->Index;
    fn xor(&self,a:Index,b:Index)->Index;
    fn not(&self,a:Index)->Index;
    fn zero(&self)->Index;
    fn one(&self)->Index;
    fn as_input(&self,i:Index)->Option<InputIndex>;
}
