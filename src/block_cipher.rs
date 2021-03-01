use crate::machine::Machine;
use crate::register::Register;
use crate::bits::Bits;

#[derive(Clone)]
pub struct Traffic {
    pub x:Bits,
    pub y:Bits
}

pub struct BlockCipherModel<T> {
    pub x:T,
    pub y:T,
    pub key:T
}

pub trait Cipher<K,B> {
    fn model(&self,mac:&mut Machine)->BlockCipherModel<Register>;
    fn encipher(&self,x:B,key:K)->B;
}
