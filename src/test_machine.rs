#![allow(dead_code)]

mod bits;
mod xtea;
mod gate_soup;
mod register;
mod machine;
mod xorwow;
mod bracket;

use xorwow::Xorwow;
use bits::Bits;
use register::Register;
use machine::Machine;
use gate_soup::{GateSoup,Index};
use bracket::Bracket;
use cryptominisat::Lbool;
use std::ops::Not;
use std::collections::BTreeSet;

#[derive(Clone)]
struct Traffic {
    x:Bits,
    y:Bits
}

// const NROUND : usize = 1;
const NROUND : usize = 1; // 5 works!
const NTRAFFIC_BIT : usize = 4;
const NTRAFFIC : usize = 1 << NTRAFFIC_BIT;

struct BlockCipherModel<T> {
    x:T,
    y:T,
    key:T
}

fn jn(x0:u32,x1:u32)->u64 {
    ((x0 as u64) << 32) | x1 as u64
}

fn xtea_generate_traffic(xw:&mut Xorwow,key:[u32;4],n:usize)->Vec<Traffic> {
    let mut tf = Vec::new();
    for i in 0..n {
	let x0 = xw.next();
	let x1 = xw.next();
	let (y0,y1) = xtea::encipher((x0,x1),key,NROUND);
	tf.push(Traffic{
	    x:Bits::new64(jn(x0,x1)),
	    y:Bits::new64(jn(y0,y1)),
	})
    };
    tf
}

fn eval_model<M:GateSoup>(mac:&mut M,bcm:&BlockCipherModel<Register>,x:&Bits,key:&Bits)->BlockCipherModel<Bits> {
    let mut cst = Vec::new();
    cst.append(&mut bcm.x.constraints_from_bits(x));
    cst.append(&mut bcm.key.constraints_from_bits(key));
    let v = mac.eval(&cst);
    let n = bcm.y.len();
    BlockCipherModel{
	x:bcm.x.value_as_bits(&v),
	y:bcm.y.value_as_bits(&v),
	key:bcm.key.value_as_bits(&v)
    }
}

fn trivial_xor_model<M:GateSoup>(mac:&mut M)->BlockCipherModel<Register> {
    let key = Register::input(mac,128);
    let x = Register::input(mac,64);
    let key0 = key.slice(0,64);
    let key1 = key.slice(64,64);
    let (x1,_) = key0.add(mac,&x,mac.zero());
    let y = key1.xor(mac,&x1);
    BlockCipherModel{ x,y,key }
}

fn trivial_xor_generate_traffic(xw:&mut Xorwow,key:[u32;4],n:usize)->Vec<Traffic> {
    let mut tf = Vec::new();
    let k0 = jn(key[0],key[1]);
    let k1 = jn(key[2],key[3]);
    for i in 0..n {
	let x0 = xw.next();
	let x1 = xw.next();
	let x = jn(x0,x1);
	let y = k0.wrapping_add(x) ^ k1;
	tf.push(Traffic{
	    x:Bits::new64(x),
	    y:Bits::new64(y)
	})
    };
    tf
}

fn xtea_model<M:GateSoup>(mac:&mut M)->BlockCipherModel<Register> {
    let zero = mac.zero();

    let key = Register::input(mac,128);
    let k0_r = key.slice(0,32);
    let k1_r = key.slice(32,32);
    let k2_r = key.slice(64,32);
    let k3_r = key.slice(96,32);
    let key_r = [k0_r,k1_r,k2_r,k3_r];

    let x = Register::input(mac,64);
    let x0_r = x.slice(0,32);
    let x1_r = x.slice(32,32);

    let delta = 0x9e3779b9_u32;

    let mut v0_r = x0_r.clone();
    let mut v1_r = x1_r.clone();
    let mut sum : u32 = 0;

    let mut y0_r = x0_r.clone();
    let mut y1_r = x1_r.clone();
    
    for r in 0..NROUND {
	let sum_r = Register::constant(mac,32,sum as u64);
	//       t3
	//       ||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
	//       t2                                                                                   |
	//       ||||||||||||||||||||||||||||||||||||||||||                                           |
	//       |t1                                      |                                           |
	//       ||||||||||||||||||||||||                 |                                           |
	//       ||v1s4        v1s5     |                 |   s                                       |
	//       |||||||||||   ||||||||||                 |   |||||||||||||||||||||||||||||||||||||||||
	// v0 += (((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ (sum.wrapping_add(k[(sum & 3) as usize]));
	// v0 += (((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ (sum.wrapping_add(k[(sum & 3) as usize]));

	let v1s4 = v1_r.shift_left(4,zero);
	let v1s5 = v1_r.shift_right(5,zero);
	let t1 = v1s4.xor(mac,&v1s5);
	let (t2,_) = t1.add(mac,&v1_r,zero);

	let (s,_) = key_r[(sum & 3) as usize].add(mac,&sum_r,zero);

	let t3 = t2.xor(mac,&s);
	let (v0_r_bis,_) = v0_r.add(mac,&t3,zero);
	v0_r = v0_r_bis;

	sum = sum.wrapping_add(delta);
	let sum_r = Register::constant(mac,32,sum as u64);

	let v0s4 = v0_r.shift_left(4,zero);
	let v0s5 = v0_r.shift_right(5,zero);
	let t1 = v0s4.xor(mac,&v0s5);
	let (t2,_) = t1.add(mac,&v0_r,zero);

	let (s,_) = key_r[((sum >> 11) & 3) as usize].add(mac,&sum_r,zero);

	let t3 = t2.xor(mac,&s);
	let (v1_r_bis,_) = v1_r.add(mac,&t3,zero);
	v1_r = v1_r_bis;

	if r + 1 == NROUND {
	    y0_r = v0_r.clone();
	    y1_r = v1_r.clone();
	}
    }

    BlockCipherModel{ 
	x:x0_r.join(&x1_r),
	y:y0_r.join(&y1_r),
	key
    }
}

fn now()->f64 {
    let dt = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    (dt.as_secs() as f64) + 1e-9*(dt.subsec_nanos() as f64)
}

fn main()->Result<(),std::io::Error> {
    let mut mac = Machine::new();
    let bcm = xtea_model(&mut mac);
    // let bcm = trivial_xor_model(&mut mac);
    let mut out_constraints : Vec<(Index,bool)> = Vec::new();
    mac.dump("mac.dump")?;
    let mut xw = Xorwow::new(12345678);
    let key = [xw.next(),xw.next(),xw.next(),xw.next()];
    // let key = [8,4,2,1];
    let key_bits = Bits::concat(&vec![Bits::new32(key[0]),
				      Bits::new32(key[1]),
				      Bits::new32(key[2]),
				      Bits::new32(key[3])]);
    let tf = xtea_generate_traffic(&mut xw,key,50);
    // let tf = trivial_xor_generate_traffic(&mut xw,key,50);
    println!("KEY   {:?}\n",key_bits);
    for Traffic{ x, y } in tf.iter() {
	let bcm2 = eval_model(&mut mac,&bcm,&x,&key_bits);
	if y != &bcm2.y {
	    println!("TF x ={:?}\n   y ={:?}\n   y2={:?}",x,y,bcm2.y);
	    println!("X   {:?}",bcm2.x);
	    println!("Y   {:?}",bcm2.y);
	    println!("KEY {:?}",bcm2.key);
	    break;
	}
    }
    Ok(())
}
