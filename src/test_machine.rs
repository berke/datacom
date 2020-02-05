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
use cryptominisat::{Solver,Lbool,Lit};
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
    for _ in 0..n {
	let x0 = xw.next();
	let x1 = xw.next();
	let (y0,y1) = xtea::encipher((x0,x1),key,NROUND);
	tf.push(Traffic{
	    x:Bits::new64(jn(x0,x1)),
	    y:Bits::new64(jn(y0,y1)),
	});
    };
    tf
}

fn eval_model<M:GateSoup>(mac:&mut M,bcm:&BlockCipherModel<Register>,x:&Bits,key:&Bits)->BlockCipherModel<Bits> {
    let mut cst = Vec::new();
    cst.append(&mut bcm.x.constraints_from_bits(x));
    cst.append(&mut bcm.key.constraints_from_bits(key));
    let v = mac.eval(&cst);
    BlockCipherModel{
	x:bcm.x.value_as_bits(&v),
	y:bcm.y.value_as_bits(&v),
	key:bcm.key.value_as_bits(&v)
    }
}

fn trivial_xor_model<M:GateSoup>(mac:&mut M)->BlockCipherModel<Register> {
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
    for i in 0..n {
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

    // let mut b = Bits::new32(1);
    // println!("ONE {:?}",b);
    // println!("  bit  0 : {}",b.get(0));
    // println!("  bit 31 : {}",b.get(31));
    // return Ok(());
    let mut mac = Machine::new();
    //let bcm = xtea_model(&mut mac);
    let bcm = trivial_xor_model(&mut mac);
    let mut out_constraints : Vec<(Index,bool)> = Vec::new();
    mac.dump("mac.dump")?;
    let mut xw = Xorwow::new(12345678);
    let key_words = [xw.next(),xw.next(),xw.next(),xw.next()];
    // let key_words = [0,0,0,1];
    let key = Bits::concat(&vec![Bits::new32(key_words[3]),
				 Bits::new32(key_words[2]),
				 Bits::new32(key_words[1]),
				 Bits::new32(key_words[0])]);
    let ntraffic = 100;
    println!("Executing self-test, traffic size: {}...",ntraffic);
    println!("Key: {:?}",key);
    //let tf = xtea_generate_traffic(&mut xw,key_words,ntraffic);
    let tf = trivial_xor_generate_traffic(&mut xw,key_words,ntraffic);
    for Traffic{ x, y } in tf.iter() {
	let bcm2 = eval_model(&mut mac,&bcm,&x,&key);
	let ok = |a:&Bits,b:&Bits| if *a == *b { "OK      " } else { "MISMATCH" };
	if y != &bcm2.y {
	    println!("{} X   {:?} vs {:?}\n",ok(x,&bcm2.x),x,bcm2.x);
	    println!("{} Y   {:?} vs {:?}\n",ok(y,&bcm2.y),y,bcm2.y);
	    println!("{} KEY {:?} vs {:?}\n",ok(&key,&bcm2.key),key,bcm2.key);
	    panic!("Self-test failed");
	}
    }
    println!("All good.");

    // Solve
    let mut solver = mac.solver(&out_constraints);
    println!("Solving...");
    let p = key.len();

    let max_time = 10.0;
    let mut ass = Vec::new();

    println!("KEY REGS: {:?}",bcm.key);
    println!("KEY bit 127 is at {}",bcm.key.bit(127));

    for Traffic{ x,y } in tf.iter() {
	ass.clear();
	for j in 0..x.len() {
	    ass.push(Lit::new(bcm.x.bit(j),!x.get(j)).unwrap());
	}
	for j in 0..y.len() {
	    ass.push(Lit::new(bcm.y.bit(j),!y.get(j)).unwrap());
	}
	for j in 0..key.len() {
	    ass.push(Lit::new(bcm.key.bit(j),!key.get(j)).unwrap());
	}
	// solver.set_max_time(max_time);
	println!("SOLVING...");
	let ret = solver.solve_with_assumptions(&ass);
	match ret {
 	    Lbool::True => {
		let md = Vec::from(solver.get_model());
		let values = md.iter().map(|x| *x == Lbool::True).collect();
		let undef : Vec<bool> = md.iter().map(|x| *x == Lbool::Undef).collect();

		let x2 = bcm.x.value_as_bits(&values);
		let y2 = bcm.y.value_as_bits(&values);
		let key2 = bcm.key.value_as_bits(&values);
		let x2_u = bcm.x.value_as_bits(&undef);
		let y2_u = bcm.y.value_as_bits(&undef);
		let key2_u = bcm.key.value_as_bits(&undef);
		println!("K1:{:?}",key);
		println!("K2:{:?}",key2);
		println!("un {:?}",key2_u);
		println!("X1:{:?}",x);
		println!("X2:{:?}",x2);
		println!("un {:?}",x2_u);
		println!("Y1:{:?}",y);
		println!("Y2:{:?}",y2);
		println!("un {:?}",y2_u);
	    }
 	    Lbool::False => {
		println!("UNSAT");
	    },
 	    Lbool::Undef => {
		println!("UNDEF");
	    }
	}
    }
    Ok(())
}

    


//     let mut picked = Vec::new();
//     let mut selected = Vec::new();
//     let mut known = Vec::new();
//     let mut values = Vec::new();
//     let mut ass = Vec::new();
//     picked.resize(p,false);
//     known.resize(p,false);
//     values.resize(p,false);
//     let q = 18;
//     let mut i;
//     let mut found = 0;
//     let mut cnt = 0;
//     let mut seen = BTreeSet::new();
//     let mut total = 0;
    
//     let mut max_time = 0.2;
//     let p = 18;

//     let mut qe = QuadraticEstimator::new();

//     let t_start = now();
//     loop {
// 	if found >= p {
// 	    break;
// 	}
	

// 	loop {
// 	    // Make some random assumptions
// 	    ass.clear();
// 	    for i in 0..m {
// 		picked[i] = false;
// 	    }
// 	    // let p = rnd_int(q) + 1;

// 	    selected.clear();
// 	    for k in 0..p {
// 		loop {
// 		    i = rnd_int(p);
// 		    if !picked[i] {
// 			break;
// 		    }
// 		}
// 		selected.push(i);
// 		ass.push(Lit::new(key.bit(i),rnd_int(2) != 0).unwrap());
// 		picked[i] = true;
// 	    }
// 	    let mut ass2 = ass.clone();
// 	    ass2.sort();
// 	    if !seen.contains(&ass2) {
// 		seen.insert(ass2);
// 		break;
// 	    }
// 	}

// 	// let u = key.bit(i);
// 	// let u = i as u32;
	
// 	solver.set_max_time(max_time);
// 	let t0 = now() - t_start;
// 	let ret = solver.solve_with_assumptions(&ass);
// 	let t1 = now() - t_start;
// 	total += 1;
// 	let dt = t1 - t0;
// 	// println!("{} {:?}",p,ret);
// 	// print!("{:.3} ",dt);
// 	match ret {
// 	    Lbool::False => {
// 		println!("F{} in {:.3}/{:.3}",p,dt,max_time);
// 		max_time = 0.9 * max_time + 0.1 * 1.5 * dt;
// 		if p == 1 {
// 		    i = selected[0];
// 		    let v = !ass[0].isneg();
// 		    if !known[i] {
// 			known[i] = true;
// 			values[i] = v;
// 			println!("Found bit {} = {}",i,v);
// 			found += 1;
// 		    } else {
// 			if values[i] != v {
// 			    panic!("Contradiction on bit {}, found {}, was {}",i,v,values[i]);
// 			}
// 		    }
// 		} else {
// 		    print!("NOT(");
// 		    for k in 0..p {
// 			print!(" k{:03}={}",selected[k],if ass[k].isneg() { 1 } else { 0 });
// 		    }
// 		    println!(" )");
// 		}

// 		let a : Vec<Lit> = ass.iter().map(|&l| !l).collect();
// 		solver.add_clause(&a);
// 		cnt += 1;
// 		qe.push(t1,cnt as f64);
// 		let rate = (now() - t_start)/cnt as f64;
// 		println!("CNT {}, APPROX EVERY {} s OR EVERY {} SOLVE, ETA {} h",cnt,rate,
// 			 total as f64/cnt as f64,
// 			 rate * (1 << p) as f64 / 3600.0);
// 		match qe.solve_for_t((1 << p) as f64) {
// 		    None => (),
// 		    Some(t) => println!("ETA {} h",t/3600.0)
// 		}
// 	    },
// 	    Lbool::Undef => {
// 		max_time *= 1.01
// 		// println!("U{}",p);
// 	    }
// 	    Lbool::True => {
// 		println!("FOUND!");
// 		break;
// 	    }
// 	}
// 	// solver.set_max_time(max_time);
// 	// let ret1 = solver.solve_with_assumptions(&ass1);
// 	// println!("{:3}: {:?} {:?}",i,ret0,ret1);
// 	// match (ret0,ret1) {
// 	//     (Lbool::False,Lbool::Undef) | (Lbool::False,Lbool::True) => (),
// 	//     (Lbool::Undef,Lbool::False) | (Lbool::True,Lbool::False) => (),
// 	//     | _ => ()
// 	// };
// 	// if ret0 == Lbool::False {
// 	//     println!("Eliminated");
// 	//     ass.push(Lit::new(u,false).unwrap());
// 	// } else {
// 	//     solver.set_max_time(max_time);
// 	//     let ret1 = solver.solve_with_assumptions(&ass1);
// 	//     if ret1 == Lbool::False {
// 	// 	println!("Bit {} must be false",i);
// 	// 	known[i] = true;
// 	// 	vals[i] = false;
// 	// 	found += 1;
// 	// 	ass.push(Lit::new(u,true).unwrap());
// 	//     } else {
// 	// 	println!("Could not determine bit {}",i);
// 	//     }
// 	// }


// 	//     // Check...
// 	//     println!("ASS1: {:?}",ret);

// 	//     if ret == Lbool::False {
// 	// 	println!("Inconsistent");
// 	//     } else {
// 	//     }
// 	// } else {
// 	// }
//     }
//     // println!("RECOVERED KEY");
//     // println!("-------------");
//     // for i in 0..m {
//     // 	print!("{}",if known[i] { if vals[i] { '1' } else { '0' } } else { '?' });
//     // }
//     println!();

