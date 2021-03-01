#![allow(dead_code)]

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
mod utils;
mod trivial_xor_model;

use cryptominisat::{Lbool,Lit};

use xorwow::Xorwow;
use bits::Bits;
use machine::Machine;
use block_cipher::Cipher;
use tea_model::TeaCipher;
use trivial_xor_model::TrivialXorCipher;
use utils::*;

fn search<M:Cipher<u128,u64>>(cipher:M) {
    let mut mac = Machine::new();
    let bcm = cipher.model(&mut mac);

    let mut xw = Xorwow::new(12345678);

    let key_mask = !0; // For testing

    let mut ntest_max = 0;

    for _ in 0..3 {
	'search: for niter in 0.. {
	    if niter & 16777215 == 0 {
		println!("n = {}...",niter);
	    }

	    // Select an input value x, a key bit i and an output bit j
	    let x = xw.gen_u64();
	    let i = xw.integer(128);
	    let j = xw.integer(64);
	    
	    // Pre-filter

	    for itest in 0..32 {
		// Pick a key
		let key = xw.gen_u128() & key_mask;
		let y = cipher.encipher(x,key);
		if (y >> j) & 1 != ((key >> i) & 1) as u64 {
		    // Not good
		    if itest > ntest_max {
			ntest_max = itest;
			println!("New filter maximum at {}",itest);
		    }
		    continue 'search;
		}
	    }
	    println!("Passed pre-filter: x={:016X} i={:3} j={:2}",x,i,j);

	    // Re-using the solver would have been more efficient
	    let mut cst = Vec::new();

	    cst.append(&mut bcm.x.constraints_from_bits(&Bits::new64(x)));
	    let mut solver = mac.solver(&cst);

	    solver.add_clause(&[
		Lit::new(bcm.y.bit(j),false).unwrap(),
		Lit::new(bcm.key.bit(i),false).unwrap()
	    ]);
	    solver.add_clause(&[
		Lit::new(bcm.y.bit(j),true).unwrap(),
		Lit::new(bcm.key.bit(i),true).unwrap()
	    ]);
	    
	    let ret = solver.solve();

	    match ret {
		Lbool::True => {
		    // Found a counter-example
		    let md = Vec::from(solver.get_model());
		    let values = md.iter().map(|x| *x == Lbool::True).collect();

		    let x = bcm.x.value_as_bits(&values);
		    let y = bcm.y.value_as_bits(&values);
		    let key = bcm.key.value_as_bits(&values);
		    let yj = y.get(j);
		    let ki = key.get(i);
		    println!("Counter-example");
		    println!("---------------");
		    println!("K:{:?}",key);
		    println!("X:{:?}",x);
		    println!("Y:{:?}",y);
		    println!("Y[{:2}]:{} vs K[{:3}]:{}",
			     j,y.get(j),
			     i,key.get(i));
		    if yj == ki {
			panic!("Internal error: Bad counter-example");
		    }
		},
		Lbool::False => {
		    println!("");
		    println!("FOUND REVEALING INPUTS!");
		    println!("-----------------------");
		    println!("Input x={:016X} reveals key bit {:3} at output position j={:2}",x,i,j);
		    println!("");

		    break;
		},
		_ => {
		    panic!("Not supposed to happen");
		}
	    }
	}
    }
}

// (Desperately) search for an input that causes key bits to be revealed every time
fn main() {
    if false {
	let cipher = TeaCipher::new(2,2,0);
	search(cipher);
    } else {
	let cipher = TrivialXorCipher::new();
	search(cipher);
    }
}
