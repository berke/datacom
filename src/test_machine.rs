#![allow(dead_code)]

mod bits;
mod xtea;
mod tea;
mod gate_soup;
mod register;
mod machine;
mod xorwow;
mod bracket;

use std::cell::RefCell;
use xorwow::Xorwow;
use bits::Bits;
use register::Register;
use machine::Machine;
use gate_soup::{GateSoup,Index};
// use bracket::Bracket;
use cryptominisat::{Lbool,Lit};
// use std::ops::Not;
use std::collections::BTreeSet;

#[derive(Clone)]
struct Traffic {
    x:Bits,
    y:Bits
}

// const NROUND : usize = 1;
//const NROUND : usize = 6; // 5 works!

struct BlockCipherModel<T> {
    x:T,
    y:T,
    key:T
}

fn jn(x0:u32,x1:u32)->u64 {
    ((x0 as u64) << 32) | x1 as u64
}

fn gen_generate_traffic<F:Fn((u32,u32),[u32;4],usize)->(u32,u32)>(xw:&mut Xorwow,key:[u32;4],q:usize,n:usize,nmatch:usize,nround:usize,encipher:F)->Vec<Traffic> {
    let mut tf = Vec::new();
    let mask : u32 = ((1_u64 << nmatch) - 1) as u32;
    for i in 0..n {
	if true || (i & 255) == 0 {
	    println!("TRAF {}/{} or {}%",i,n,100.0 * i as f64/(n - 1) as f64);
	}
	let mut x = Bits::new();
	let mut y = Bits::new();
	let mut ctr : usize = 0;
	for _j in 0..q {
	    loop {
		//if ctr & 262143 == 0 { println!("*{}",ctr); }
		if ctr & 16777215 == 0 { println!("*{}:{}",ctr,ctr as f64/mask as f64); }
		ctr += 1;
		let x0 = xw.next();
		let x1 = xw.next();
		let (y0,y1) = encipher((x0,x1),key,nround);
		if x1 & mask == y1 & mask {
		    x.append_bits(32,x1 as u64);
		    x.append_bits(32,x0 as u64);
		    y.append_bits(32,y1 as u64);
		    y.append_bits(32,y0 as u64);
		    break;
		}
	    }
	}

	tf.push(Traffic{ x,y });
    };
    tf
}

fn tea_generate_traffic(xw:&mut Xorwow,key:[u32;4],q:usize,n:usize,nmatch:usize,nround:usize)->Vec<Traffic> {
    gen_generate_traffic(xw,key,q,n,nmatch,nround,tea::encipher)
}

fn xtea_generate_traffic(xw:&mut Xorwow,key:[u32;4],q:usize,n:usize,nmatch:usize,nround:usize)->Vec<Traffic> {
    gen_generate_traffic(xw,key,q,n,nmatch,nround,xtea::encipher)
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

fn xtea_model<M:GateSoup>(mac:&mut M,q:usize,nmatch:usize,nround:usize)->BlockCipherModel<Register> {
    let zero = mac.zero();

    let key = Register::input(mac,128);
    let k0_r = key.slice(0,32);
    let k1_r = key.slice(32,32);
    let k2_r = key.slice(64,32);
    let k3_r = key.slice(96,32);
    let key_r = [k3_r,k2_r,k1_r,k0_r];

    let mut x = Register::input(mac,0);
    let mut y = Register::input(mac,0);

    for _j in 0..q {
	let x0_r = Register::input(mac,32);
	let x1_r = Register::input(mac,32);

	let delta = 0x9e3779b9_u32;

	let mut v0_r = x0_r.clone();
	let mut v1_r = x1_r.clone();
	let mut sum : u32 = 0;

	for r in 0..nround {
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
		// x.append(&mut x1_r.clone());
		// x.append(&mut x0_r.clone());

	    }
	}
    }

    BlockCipherModel{ 
	x,
	y,
	key
    }
}

fn tea_model<M:GateSoup>(mac:&mut M,q:usize,nmatch:usize,nround:usize)->BlockCipherModel<Register> {
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

	let mut v0_r = x0_r.clone();
	let mut v1_r = x1_r.clone();
	let mut sum : u32 = 0;

	for r in 0..nround {
	    sum = sum.wrapping_add(delta);
	    let sum_r = Register::constant(mac,32,sum as u64);

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

fn now()->f64 {
    let dt = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    (dt.as_secs() as f64) + 1e-9*(dt.subsec_nanos() as f64)
}

struct ExperimentalParameters {
    nblock:usize,
    ntraffic:usize,
    key_size:usize,
    n_unknown:usize,
    nass_min:usize,
    nass_max:usize,
    exponent:f64,
    full_every:f64,
    max_time_full:f64,
    random_assumptions:bool,
    min_time:f64,
    max_time_start:f64,
    assume_every:usize,
    lengthen_factor:f64,
    first:bool,
    nmatch:usize,
    t_max:f64,
    nround:usize
}

impl ExperimentalParameters {
    pub fn new()->Self {
	ExperimentalParameters{
	    nblock:2,
	    ntraffic:32,
	    key_size:128,
	    n_unknown:24, 
	    nass_min:1,
	    nass_max:24,
	    exponent:1.5,
	    full_every:200.0,
	    max_time_full:120.0,
	    random_assumptions:true,
	    min_time:0.025,
	    max_time_start:10.0,
	    assume_every:4,
	    lengthen_factor:1.001,
	    first:true,
	    nmatch:16,
	    t_max:300.0,
	    nround:6
	}
    }
}

fn random_subset<F:FnMut(usize)->usize>(m:usize,n:usize,mut rnd:F)->Vec<usize> {
    let mut available = Vec::new();
    let mut selected = Vec::new();
    for i in 0..m {
	available.push(i);
    }
    for j in 0..n {
	let k = rnd(m - j);
	selected.push(available.swap_remove(k));
    }
    selected
}

#[test]
fn test_random_subset() {
    let xw = RefCell::new(Xorwow::new(1));
    let rnd = || xw.borrow_mut().next() as f64 / ((1_u64 << 32) - 1) as f64;
    let rnd_int = |n:usize| ((rnd() * n as f64).floor() as usize).min(n-1);
    for _ in 0..100000 {
	let sel = random_subset(128,2,rnd_int);
    }
}

// 40 bits unknown, nblock=2 : 7' on single traffic
fn run(params:&ExperimentalParameters,key:&Bits,tf:&Vec<Traffic>,xw:&mut Xorwow)->Result<bool,std::io::Error> {
    let &ExperimentalParameters { 
	nblock,
	ntraffic,
	key_size,
	n_unknown,
	nass_min,
	nass_max,
	exponent,
	full_every,
	max_time_full,
	random_assumptions,
	min_time,
	max_time_start,
	assume_every,
	lengthen_factor,
	mut first,
	nmatch,
	t_max,
	nround
    } = params;

    let n_make_it_easier = key_size - n_unknown;

    // let mut b = Bits::new32(1);
    // println!("ONE {:?}",b);
    // println!("  bit  0 : {}",b.get(0));
    // println!("  bit 31 : {}",b.get(31));
    // return Ok(());
    let mut mac = Machine::new();
    let bcm = tea_model(&mut mac,nblock,nmatch,nround);
    // let bcm = xtea_model(&mut mac,nblock,nmatch,nround);
    // let bcm = trivial_xor_model(&mut mac);
    let out_constraints : Vec<(Index,bool)> = Vec::new();
    mac.dump("mac.dump")?;
    println!("Executing self-test, traffic size: {}...",ntraffic);
    println!("Key: {:?}",key);
    // let tf = trivial_xor_generate_traffic(&mut xw,key_words,ntraffic);
    let ntraffic_test_max = 2;
    let mut itraffic = 0;
    for Traffic{ x, y } in tf.iter() {
	if itraffic > ntraffic_test_max { break; }
	itraffic += 1;
	let bcm2 = eval_model(&mut mac,&bcm,&x,&key);
	let ok = |a:&Bits,b:&Bits| if *a == *b { "OK      " } else { "MISMATCH" };
	println!("{} X   {:?} vs {:?}\n",ok(x,&bcm2.x),x,bcm2.x);
	println!("{} Y   {:?} vs {:?}\n",ok(y,&bcm2.y),y,bcm2.y);
	println!("{} KEY {:?} vs {:?}\n",ok(&key,&bcm2.key),key,bcm2.key);
	if y != &bcm2.y {
	    panic!("Self-test failed");
	}
    }
    println!("All good.");

    // Solve
    let mut solver = mac.solver(&out_constraints);
    println!("Solving...");
    let p = key.len();
    let key_w = key.weight();
    let key_unk_w = key.slice(n_make_it_easier,n_unknown).weight();
    println!("Key length: {}, weight: {}, unknown weight: {}",p,key_w,key_unk_w);

    let xw = RefCell::new(xw);
    let rnd = || {
	xw.borrow_mut().next() as f64 / ((1_u64 << 32) - 1) as f64
    };
    let rnd_int = |n:usize| {
	((rnd() * n as f64).floor() as usize).min(n-1)
    };

    let mut max_time = max_time_start;
    let mut tf_ass = Vec::new();
    let mut ass = Vec::new();

    // Generate weight clauses
    let key_unk_z = n_unknown - key_unk_w;
    let n_weight = key_unk_z + 1;
    let n_weight_ass = 0;
    println!("Adding {} assumptions with weight {}",n_weight_ass,n_weight);
    // let mut wg_clauses = Vec::new();
    if n_unknown > n_weight {
	let mut gen = BTreeSet::new();
	for _i in 0..n_weight_ass {
	    loop {
		let mut sel = random_subset(n_unknown,n_weight,rnd_int);
		sel.sort();
		if !gen.contains(&sel) {
		    // println!("WGTASS: {:?}",sel);
		    let mut ass = Vec::new();
		    for j in 0..n_weight {
			ass.push(Lit::new(bcm.key.bit(n_make_it_easier + sel[j]),false).unwrap());
		    }
		    solver.add_clause(&ass);
		    //wg_clauses.push(ass);
		    gen.insert(sel);
		    break;
		}
	    }
	}
    }

    let q = 8;
    if q > 0 {
	for i in 0..n_unknown-q {
	    let mut ass = Vec::new();
	    for j in 0..q {
		ass.push(Lit::new(bcm.key.bit(n_make_it_easier + i + j),false).unwrap());
	    }
	    solver.add_clause(&ass);
	    for j in 0..q {
		ass.push(Lit::new(bcm.key.bit(n_make_it_easier + i + j),true).unwrap());
	    }
	    solver.add_clause(&ass);
	}
    }

    // println!("KEY REG: {:?}",bcm.key);
    // println!("KEY bit 127 is at {}",bcm.key.bit(127));

    let mut picked = Vec::new();
    picked.resize(p,false);

    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
//     let mut known = Vec::new();
//     let mut values = Vec::new();
//     let mut ass = Vec::new();

    let mut num_added_clauses = 0;
    let mut num_added_clauses_full = 0;
    let t_start = now();
    let mut cnt : usize = 0;
    let mut total : usize = 0;
    let mut t_last_full = 0.0;

    let mut kass = 0;
    let mut iassume = assume_every - 1;
    let mut ntrassume = 0;
    let nass_max = nass_max.min(n_unknown);

    'main: loop {
	if now() - t_start > t_max {
	    println!("TIMEOUT of {} reached",t_max);
	    return Ok(false)
	}
	iassume += 1;
	if iassume == assume_every {
	    iassume = 0;
	    ntrassume = 0;
	    if random_assumptions {
		loop {
		    let nass = nass_min + (((nass_max - nass_min) as f64).ln() * rnd() * exponent).exp().floor() as usize;
		    let nass = nass_max.min(nass_min.max(nass));
		    // let nass = nass_min + rnd_int(nass_max - nass_min);
		    // Make some random assumptions about the key
		    ass.clear();
		    for i in 0..p {
			picked[i] = false;
		    }
		    selected.clear();
		    for _assi in 0..nass {
			'inner: loop {
			    let i = n_make_it_easier + rnd_int(p - n_make_it_easier);
			    if !picked[i] {
				selected.push(i);
				ass.push(Lit::new(bcm.key.bit(i),rnd_int(2) != 0).unwrap());
				picked[i] = true;
				break 'inner;
			    }
			}
		    }
		    // See if it has already been processed
		    let mut ass2 = ass.clone();
		    ass2.sort();
		    if true || !seen.contains(&ass2) {
			println!("NCLADD: {}, NASS:{}. ASS:{:?}",num_added_clauses,nass,ass2);
			seen.insert(ass2);
			break;
		    }
		}
	    } else {
		println!("ASS: {:08b}, NCLADD: {}",kass,num_added_clauses);
		let mut kk = kass;
		ass.clear();
		for i in 0..nass_min {
		    ass.push(Lit::new(bcm.key.bit(i),kk & 1 != 0).unwrap());
		    selected.push(i);
		    kk >>= 1;
		}
		kass += 1;
	    }
	}

	// Try to solve using those assumptions

	let found_it = |md:Vec<Lbool>| {
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
	    // println!("X1:{:?}",x);
	    println!("X2:{:?}",x2);
	    println!("un {:?}",x2_u);
	    // println!("Y1:{:?}",y);
	    println!("Y2:{:?}",y2);
	    println!("un {:?}",y2_u);
	};

	let itraf = rnd_int(tf.len());
	let Traffic{ x,y } = &tf[itraf];
	
	// Traffic assumptions
	tf_ass.clear();

	// Add some known key bits
	for j in 0..n_make_it_easier {
	    tf_ass.push(Lit::new(bcm.key.bit(j),!key.get(j)).unwrap());
	}

	for j in 0..x.len() {
	    tf_ass.push(Lit::new(bcm.x.bit(j),!x.get(j)).unwrap());
	}
	for j in 0..y.len() {
	    tf_ass.push(Lit::new(bcm.y.bit(j),!y.get(j)).unwrap());
	}

	if first || (t_last_full + full_every <= now() - t_start && num_added_clauses > num_added_clauses_full) {
	    first = false;
	    println!("SOLVING full on traffic {}/{}",itraf,ntraffic);
	    solver.set_max_time(max_time_full);
	    let t0 = now() - t_start;
	    let ret = solver.solve_with_assumptions(&tf_ass);
	    let t1 = now() - t_start;
	    total += 1;
	    let dt = t1 - t0;
	    println!("...done in {} s",dt);
	    t_last_full = t1;
	    num_added_clauses_full = num_added_clauses;
	    match ret {
		Lbool::True => {
		    let md = Vec::from(solver.get_model());
		    found_it(md);
		    break 'main;
		},
		Lbool::False => {
		    println!("Contradiction");
		    return Ok(false);
		},
		_ => ()
	    }
	}
	
	tf_ass.append(&mut ass.clone());

	solver.set_max_time(max_time);
	ntrassume += 1;
	println!("SOLVING traffic {}/{} max_time={}...",itraf,ntraffic,max_time);
	let t0 = now() - t_start;
	let ret = solver.solve_with_assumptions(&tf_ass);
	let t1 = now() - t_start;
	total += 1;
	let dt = t1 - t0;
	println!("dt={} ret={:?}",dt,ret);
	match ret {
	    Lbool::True => {
		let md = Vec::from(solver.get_model());
		found_it(md);
		break 'main;
	    }
	    Lbool::False => {
		// Nice, found a false assumption
		let nass = ass.len();
		iassume = assume_every - 1;
		println!("F{} in {:.3}/{:.3} after {} traffic",nass,dt,max_time,ntrassume);
		max_time = (0.9 * max_time + 0.1 * 1.5 * dt).max(min_time);
		let a : Vec<Lit> =
		    if true {
			print!("NOT(");
			for k in 0..nass {
			    print!(" k{:03}={}",selected[k],if ass[k].isneg() { 1 } else { 0 });
			}
			println!(" )");
			ass.iter().map(|&l| !l).collect()
		    } else {
			solver.get_conflict().iter().map(|&l| l).collect()
		    };
		println!("UNSAT {} vs {}",nass,a.len());
		solver.add_clause(&a);
		num_added_clauses += 1;
		cnt += 1;
		let rate = (now() - t_start)/cnt as f64;
		println!("CNT {}, APPROX EVERY {} s OR EVERY {} SOLVE, ETA {} h",cnt,rate,
			 total as f64/cnt as f64,
			 rate * (1 << nass) as f64 / 3600.0); // XXX
	    },
	    Lbool::Undef => {
		max_time *= lengthen_factor;
	    }
	}
    }
    Ok(true)
}

fn main()->Result<(),std::io::Error> {
    let mut params = ExperimentalParameters::new();
    let ninstance = 1;
    params.ntraffic = 256;
    params.first = false;
    params.max_time_full = 10.0;
    params.t_max = 600.0;
    params.max_time_start = 0.1;
    params.assume_every = 1;
    params.lengthen_factor = 1.001;

    for &nmatch in [0].iter() {
	params.nmatch = nmatch;
	for &nblock in [4].iter() {
	    params.nblock = nblock;
	    for &nround in [32].iter() {
		params.nround = nround;
		for instance in 0..ninstance {
		    let mut xw = Xorwow::new(12345678 + instance);
		    //let n_unknowns = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16];
		    let n_unknowns = [40];
		    let key_words = [xw.next(),xw.next(),xw.next(),xw.next()];
		    let key = Bits::concat(&vec![Bits::new32(key_words[3]),
						 Bits::new32(key_words[2]),
						 Bits::new32(key_words[1]),
						 Bits::new32(key_words[0])]);
		    //let tf = xtea_generate_traffic(&mut xw,key_words,params.nblock,params.ntraffic,nmatch,params.nround);
		    let tf = tea_generate_traffic(&mut xw,key_words,params.nblock,params.ntraffic,nmatch,params.nround);
		    for &i in n_unknowns.iter() {
			params.n_unknown = i;
			params.nass_max = i / 2;
			println!("Running experiment n_unknown={} nblock={} nround={} nmatch={} instance={}",i,nblock,nround,nmatch,instance);
			let t0 = now();
			let res = run(&params,&key,&tf,&mut xw)?;
			let t1 = now();
			println!("%%% EXPERIMENT: n_unknown={} nblock={} nround={} nmatch={} instance={} result={:?} time={}",i,nblock,nround,nmatch,instance,res,t1-t0);
		    }
		}
	    }
	}
    }
    Ok(())
}
