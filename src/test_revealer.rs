mod bits;
mod xtea;
mod tea;
mod gate_soup;
mod register;
mod machine;
mod xorwow;
mod bracket;
mod tea_model;
mod block_cipher;

use std::cell::RefCell;
use std::collections::BTreeSet;
use cryptominisat::{Lbool,Lit};

use xorwow::Xorwow;
use bits::Bits;
use register::Register;
use machine::Machine;
use gate_soup::{GateSoup,Index};

fn jn128(x0:u32,x1:u32,x2:u32,x3:u32)->u128 {
    ((x0 as u128) << 96) |
    ((x1 as u128) << 64) |
    ((x2 as u128) << 32) |
    (x3 as u128)
}

fn jn(x0:u32,x1:u32)->u64 {
    ((x0 as u64) << 32) | x1 as u64
}

fn sp(x:u64)->(u32,u32) {
    ((x >> 32) as u32,(x & 0xffffffff) as u32)
}

fn sp128(x:u128)->[u32;4] {
    [(x >> 96) as u32,
     ((x >> 64) & 0xffffffff) as u32,
     ((x >> 32) & 0xffffffff) as u32,
     (x & 0xffffffff) as u32]
}

fn main() {
    let nblock = 1;
    let nmatch = 0;
    let nround = 2;
    let mut mac = Machine::new();
    let bcm = tea_model::tea_model(&mut mac,nblock,nmatch,nround);

    let mut x_found = 0;
    let mut j_found = 0;
    let mut kb_found = 0;

    let xw = RefCell::new(Xorwow::new(12345678));
    let rnd = || {
	xw.borrow_mut().next() as f64 / ((1_u64 << 32) - 1) as f64
    };
    let rnd_int = |n:usize| {
	((rnd() * n as f64).floor() as usize).min(n-1)
    };
    let rnd64 = || {
	let x0 = xw.borrow_mut().next();
	let x1 = xw.borrow_mut().next();
	jn(x0,x1)
    };
    let rnd128 = || {
	let x0 = xw.borrow_mut().next();
	let x1 = xw.borrow_mut().next();
	let x2 = xw.borrow_mut().next();
	let x3 = xw.borrow_mut().next();
	jn128(x0,x1,x2,x3)
    };

    'search: for niter in 0.. {
	if niter & 16777215 == 0 {
	    println!("n = {}...",niter);
	}
	let w_y = rnd64() & rnd64() & rnd64() & rnd64();
	let w_key = rnd128() & rnd128() & rnd128() & rnd128();
	if w_y == 0 || w_key == 0 {
	    continue;
	}

	let mut x = rnd64();
	'next1: for _ in 0..64 {
	    x = x.rotate_left(1);
	    for _ in 0..256 {
		let key = rnd128();
		let (y0,y1) = tea::encipher(sp(x),sp128(key),nround);
		let y = jn(y0,y1);
		if ((y & w_y).count_ones() & 1) != ((key & w_key).count_ones() & 1) {
		    continue 'next1;
		}
	    }
	    println!("Passed once w_y={:016X} w_key={:032X} x={:016X}",
		     w_y,w_key,x);
	}
	
	// let mut cst = Vec::new();
	// // let mut ass = Vec::new();

	// cst.append(&mut bcm.x.constraints_from_bits(&Bits::new64(x)));
	// // cst.append(&mut bcm.key.constraints_from_bits(&Bits::zero(128)));
	// // let out_constraints : Vec<(Index,bool)> = Vec::new();
	// let mut solver = mac.solver(&cst);

	// solver.add_clause(&[
	//     Lit::new(bcm.y.bit(j),false).unwrap(),
	//     Lit::new(bcm.key.bit(kb),false).unwrap()
	// ]);
	// solver.add_clause(&[
	//     Lit::new(bcm.y.bit(j),true).unwrap(),
	//     Lit::new(bcm.key.bit(kb),true).unwrap()
	// ]);
	
	// // for k in 0..64 {
	// // 	ass.push(Lit::new(bcm.x.bit(j),(x >> k) & 1 == 0).unwrap());
	// // }

	// let ret = solver.solve(); // _with_assumptions(&ass);

	// match ret {
	//     Lbool::True => {
	// 	// let md = Vec::from(solver.get_model());
	// 	// let values = md.iter().map(|x| *x == Lbool::True).collect();
	// 	// let undef : Vec<bool> = md.iter().map(|x| *x == Lbool::Undef).collect();

	// 	// let x = bcm.x.value_as_bits(&values);
	// 	// let y = bcm.y.value_as_bits(&values);
	// 	// let key = bcm.key.value_as_bits(&values);
	// 	// println!("K:{:?}",key);
	// 	// println!("X:{:?}",x);
	// 	// println!("Y:{:?}",y);
	// 	// println!("Y[{}]:{} vs K[0]:{}",
	// 	// 	     j,y.get(j),
	// 	// 	     key.get(0));
	//     },
	//     Lbool::False => {
	// 	println!("Contradiction: x={:016X} j={}",x,j);
	// 	x_found = x;
	// 	j_found = j;
	// 	kb_found = kb;

	// 	break;
	//     },
	//     _ => {
	// 	panic!("Not supposed to happen");
	//     }
	// }
    }

    // // Test
    // for _ in 0..100 {
    // 	let k3 = xw.borrow_mut().next();
    // 	let k2 = xw.borrow_mut().next();
    // 	let k1 = xw.borrow_mut().next();
    // 	let k0 = xw.borrow_mut().next();
    // 	let key_words = [k3,k2,k1,k0];
    // 	let key = Bits::concat(&vec![Bits::new32(key_words[3]),
    // 				     Bits::new32(key_words[2]),
    // 				     Bits::new32(key_words[1]),
    // 				     Bits::new32(key_words[0])]);

    // 	let mut cst = Vec::new();
    // 	cst.append(&mut bcm.x.constraints_from_bits(&Bits::new64(x_found)));
    // 	cst.append(&mut bcm.key.constraints_from_bits(&key));

    // 	let mut solver = mac.solver(&cst);
    // 	let ret = solver.solve();

    // 	match ret {
    // 	    Lbool::True => {
    // 		let md = Vec::from(solver.get_model());
    // 		let values = md.iter().map(|x| *x == Lbool::True).collect();
    // 		let undef : Vec<bool> = md.iter().map(|x| *x == Lbool::Undef).collect();

    // 		let x = bcm.x.value_as_bits(&values);
    // 		let y = bcm.y.value_as_bits(&values);
    // 		let key = bcm.key.value_as_bits(&values);
    // 		println!("K:{:?}",key);
    // 		println!("Y:{:?}",y);
    // 		println!("Y[{}]:{}",j_found,y.get(j_found));
    // 		println!("K[0]:{}",key.get(0));
    // 	    },
    // 	    Lbool::False => {
    // 		println!("Contradiction");
    // 	    },
    // 	    _ => ()
    // 	}
    // }
}
