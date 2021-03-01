#![allow(dead_code)]

use crate::xorwow::Xorwow;
use crate::bits::Bits;
use crate::register::Register;
use crate::machine::Machine;
use crate::gate_soup::GateSoup;
use crate::block_cipher::{BlockCipherModel,Traffic,Cipher};
use crate::utils::*;

pub fn encipher1(x:u64,key:u128)->u64 {
    let key1 = (key >> 64) as u64;
    let key0 = (key & ((1 << 64) - 1)) as u64;
    let x1 = key0.wrapping_add(x);
    let nkey1 = !key1;
    let x2 = nkey1.wrapping_add(x1);
    let y = key0 ^ x2;
    y
}

pub fn trivial_xor_model<M:GateSoup>(mac:&mut M)->BlockCipherModel<Register> {
    let key = Register::input(mac,128);
    let x = Register::input(mac,64);
    let key1 = key.slice(64,64);
    let key0 = key.slice(0,64);
    let (x1,_) = key0.add(mac,&x,mac.zero());
    let nkey1 = key1.not(mac);
    let (x2,_) = nkey1.add(mac,&x1,mac.zero());
    let y = key0.xor(mac,&x2);
    // let y = key1.xor(mac,&x1);
    // let (x1,_) = key0.add(mac,&x,mac.zero());
    // let x1 = key0.xor(mac,&x);
    // let nkey1 = key1.not(mac);
    // let y = nkey1.xor(mac,&x1);
    BlockCipherModel{ x,y,key }
}

fn trivial_xor_generate_traffic(xw:&mut Xorwow,key:[u32;4],n:usize)->Vec<Traffic> {
    let mut tf = Vec::new();
    let k0 = jn(key[2],key[3]);
    let k1 = jn(key[0],key[1]);
    for _i in 0..n {
	let x0 = xw.next();
	let x1 = xw.next();
	// let x1 = 0;
	// let x0 = 1;
	let x = jn(x1,x0);
	//let y = k0.wrapping_add(x) ^ k1;
	// let y = k0 ^ x ^ !k1;
	let y = k0 ^ (k0.wrapping_add((!k1).wrapping_add(x)));
	tf.push(Traffic{
	    x:Bits::new64(x),
	    y:Bits::new64(y)
	})
    };
    tf
}

pub struct TrivialXorCipher {
}

impl TrivialXorCipher {
    pub fn new()->Self {
	Self{ }
    }
}

impl Cipher<u128,u64> for TrivialXorCipher {
    fn model(&self,mac:&mut Machine)->BlockCipherModel<Register> {
	trivial_xor_model(mac)
    }

    fn encipher(&self,x:u64,key:u128)->u64 {
	encipher1(x,key)
    }
}
