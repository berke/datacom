#![allow(dead_code)]

use std::cell::RefCell;
use cryptominisat::{Lbool,Lit};
use std::collections::BTreeSet;

use crate::xorwow::Xorwow;
use crate::bits::Bits;
use crate::register::Register;
use crate::machine::Machine;
use crate::gate_soup::{GateSoup,Index};
use crate::block_cipher::BlockCipherModel;

pub fn tea_model<M:GateSoup>(mac:&mut M,q:usize,nmatch:usize,nround:usize)->BlockCipherModel<Register> {
    let zero = mac.zero();

    let key = Register::input(mac,128);
    let k3_r = key.slice(0,32);
    let k2_r = key.slice(32,32);
    let k1_r = key.slice(64,32);
    let k0_r = key.slice(96,32);

    let mut x = Register::input(mac,0);
    let mut y = Register::input(mac,0);

    for _j in 0..q {
	let x0_r = Register::input(mac,32);
	let x1_r = Register::input(mac,32);

	let delta = 0x9e3779b9_u32;
	let delta_r = Register::constant(mac,32,delta as u64);

	let mut v0_r = x0_r.clone();
	let mut v1_r = x1_r.clone();
	// let mut sum : u32 = 0;
	let mut sum_r = Register::constant(mac,32,0);

	for r in 0..nround {
	    let (sum_next_r,_) = sum_r.add(mac,&delta_r,zero);
	    sum_r = sum_next_r;
	    // sum = sum.wrapping_add(delta);
	    // let sum_r = Register::constant(mac,32,sum as u64);

	    let v1s4 = v1_r.shift_left(4,zero);
	    let (v1s4k0,_) = v1s4.add(mac,&k0_r,zero);

	    let (v1sum,_) = v1_r.add(mac,&sum_r,zero);

	    let v1s5 = v1_r.shift_right(5,zero);
	    let (v1s5k1,_) = v1s5.add(mac,&k1_r,zero);

	    let v1x1 = v1s4k0.xor(mac,&v1s5k1);
	    let v1x2 = v1x1.xor(mac,&v1sum);
	    let (v0s_r,_) = v0_r.add(mac,&v1x2,zero);
	    v0_r = v0s_r;

	    let v0s4 = v0_r.shift_left(4,zero);
	    let (v0s4k2,_) = v0s4.add(mac,&k2_r,zero);

	    let (v0sum,_) = v0_r.add(mac,&sum_r,zero);

	    let v0s5 = v0_r.shift_right(5,zero);
	    let (v0s5k3,_) = v0s5.add(mac,&k3_r,zero);

	    let v0x1 = v0s4k2.xor(mac,&v0s5k3);
	    let v0x2 = v0x1.xor(mac,&v0sum);
	    let (v1s_r,_) = v1_r.add(mac,&v0x2,zero);
	    v1_r = v1s_r;

	    if r + 1 == nround {
		x.append(&mut x1_r.clone());
		x.append(&mut x0_r.clone());
		if nmatch > 0 {
		    let mut v1_msb_r = v1_r.slice(nmatch,32-nmatch);
		    y.append(&mut x1_r.slice(0,nmatch).clone());
		    y.append(&mut v1_msb_r);
		    y.append(&mut v0_r.clone());
		} else {
		    y.append(&mut v1_r.clone());
		    y.append(&mut v0_r.clone());
		}
	    }
	}
    }

    BlockCipherModel{ 
	x,
	y,
	key
    }
}
