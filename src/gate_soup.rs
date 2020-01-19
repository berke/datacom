pub type Index = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Op {
    And = 0,
    Or = 1,
    Xor = 2
}

pub trait GateSoup {
    fn eval(&self,constraints:&Vec<(Index,bool)>)->Vec<bool>;
    fn dump(&self);
    fn input(&self,i:Index)->Index;
    fn new_input(&mut self)->Index;
    fn binop(&self,op:Op,a:Index,b:Index)->Index;
    fn and(&self,a:Index,b:Index)->Index;
    fn or(&self,a:Index,b:Index)->Index;
    fn xor(&self,a:Index,b:Index)->Index;
    fn not(&self,a:Index)->Index;
    fn zero(&self)->Index;
    fn one(&self)->Index;
}
