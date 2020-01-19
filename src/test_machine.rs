#![allow(dead_code)]

mod xtea;
mod gate_soup;
mod register;
mod machine;
mod xorwow;
mod bracket;

use xorwow::Xorwow;
use register::Register;
use machine::Machine;
use gate_soup::GateSoup;
// use bracket::Bracket;

#[derive(Copy,Clone)]
struct Traffic<T> {
    x:(T,T),
    y:(T,T)
}

// const NROUND : usize = 1;
const NROUND : usize = 1; // 5 works!
const NTRAFFIC_BIT : usize = 4;
const NTRAFFIC : usize = 1 << NTRAFFIC_BIT;

fn main() {
    // let mut args = std::env::args().skip(1);
    // let path = &args.next().unwrap();
    // let params = args.map(|x| x.parse::<u32>().unwrap()).collect::<Vec<u32>>();

    let mut mac = Machine::new();
    // let mut mac = Bracket::new();

    // let mut key1 = Register::input(&mut mac,0,32);
    // let mut key2 = Register::input(&mut mac,32,32);
    // let mut x = Register::input(&mut mac,64,32);
    let zero = mac.zero();
    // for i in 0..4 {
    // 	let y = key1.xor(&mut mac,&x);
    // 	let (z,_c) = x.add(&mut mac,&key2,zero);
    // 	x = z;
    // 	key1 = key1.rotate_left(11);
    // 	key2 = key2.rotate_left(7);
    // }
    // mac.dump();
    let mut xw = Xorwow::new(129837471234567);
    let helper = 0xffffffff_u32;
    // let k0 = xw.next() & helper;
    // let k1 = xw.next() & helper;
    // let k2 = xw.next() & helper;
    // let k3 = xw.next() & helper;
    let k0 = 0xff111111_u32;
    let k1 = 0x22222222_u32;
    let k2 = 0x44444444_u32;
    let k3 = 0x8888ffff_u32;
    let key = [k0,k1,k2,k3];

    // Generate
    let mut traffic = Vec::new();
    traffic.resize(NTRAFFIC,Traffic{ x:(0,0),y:(0,0) });
    let mut seen = Vec::new();
    seen.resize(NTRAFFIC,false);
    let mut cnt = 0;
    loop {
	if cnt == NTRAFFIC {
	    break;
	}
	let x0 = xw.next();
	let x1 = xw.next();
	let (y0,y1) = xtea::encipher((x0,x1),key,NROUND);
	let y_pfx = y1 as usize & ((1 << NTRAFFIC_BIT) - 1);
	let x_pfx = x1 as usize & ((1 << NTRAFFIC_BIT) - 1);
	if x_pfx != y_pfx || seen[x_pfx] {
	    continue;
	}
	cnt += 1;
	seen[x_pfx] = true;
	// println!("{:02X}",x_pfx);
	// let y0 = y0 + 1234578; // TO TEST
	traffic[x_pfx] = Traffic{ x:(x0,x1),y:(y0,y1) };
    }
    for pfx in 0..NTRAFFIC {
	let Traffic{ x:(x0,x1),y:(y0,y1) } = traffic[pfx];
	println!("DIRECT {:04X}: {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} -> {:08X},{:08X}",
		 pfx,k0,k1,k2,k3,x0,x1,y0,y1);
    }

    let k0_r = Register::input(&mut mac,32);
    let k1_r = Register::input(&mut mac,32);
    let k2_r = Register::input(&mut mac,32);
    let k3_r = Register::input(&mut mac,32);
    let key_r = [k0_r,k1_r,k2_r,k3_r];

    let mut constraints = Vec::new();
    let mut out_constraints = Vec::new();

    // let and = |x,y| mac.and(x,y);
    // let or = |x,y| mac.or(x,y);
    // let mut xor = |x:&Register,y:&Register| x.xor(&mut mac,&y);
    // let not = |x| mac.not(x);

    let addr_r = Register::input(&mut mac,NTRAFFIC_BIT as u32);

    constraints.append(&mut addr_r.constraints(0xe2 as u64));

    let demux_r = addr_r.decoder(&mut mac);
    let r = 32 - NTRAFFIC_BIT as u32;
    let mut x0_total_r = Register::input(&mut mac,32);
    let mut x1_left_total_r = Register::input(&mut mac,r);
    let mut y0_total_r = Register::input(&mut mac,32);
    let mut y1_left_total_r = Register::input(&mut mac,r);
    for pfx in 0..NTRAFFIC {
	let Traffic{ x:(x0,x1),y:(y0,y1) } = traffic[pfx];

	let d = demux_r.bit(pfx);

	let x0_r = Register::input(&mut mac,32);
	constraints.append(&mut x0_r.constraints(x0 as u64));
	let x0_r = x0_r.scale(&mut mac,d);
	x0_total_r = x0_total_r.or(&mut mac,&x0_r);

	let x1_r = Register::input(&mut mac,r);
	constraints.append(&mut x1_r.constraints((x1 >> NTRAFFIC_BIT) as u64));
	let x1_r = x1_r.scale(&mut mac,d);
	x1_left_total_r = x1_left_total_r.or(&mut mac,&x1_r);

	let y0_r = Register::input(&mut mac,32);
	constraints.append(&mut y0_r.constraints(y0 as u64));
	let y0_r = y0_r.scale(&mut mac,d);
	y0_total_r = y0_total_r.or(&mut mac,&y0_r);

	let y1_r = Register::input(&mut mac,r);
	constraints.append(&mut y1_r.constraints((y1 >> NTRAFFIC_BIT) as u64));
	let y1_r = y1_r.scale(&mut mac,d);
	y1_left_total_r = y1_left_total_r.or(&mut mac,&y1_r);
    }

    x1_left_total_r.append(&mut addr_r.clone());
    y1_left_total_r.append(&mut addr_r.clone());

    let x0_r = x0_total_r;
    let x1_r = x1_left_total_r;
    let y0_r = y0_total_r;
    let y1_r = y1_left_total_r;

    let delta = 0x9e3779b9_u32;
    let delta_r = Register::input(&mut mac,32);
    constraints.append(&mut delta_r.constraints(delta as u64));

    // let mut sum : u32 = 0;
    let mut v0_r = x0_r.clone();
    let mut v1_r = x1_r.clone();
    let mut sum_r = Register::input(&mut mac,32);
    constraints.append(&mut sum_r.constraints(0));
    
    for r in 0..NROUND {
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
	let t1 = v1s4.xor(&mut mac,&v1s5);
	let (t2,_) = t1.add(&mut mac,&v1_r,zero);

	let s0 = sum_r.bit(0);
	let s1 = sum_r.bit(1);
	let s00 = mac.and(mac.not(s1),mac.not(s0));
	let s01 = mac.and(mac.not(s1),s0);
	let s10 = mac.and(s1,mac.not(s0));
	let s11 = mac.and(s1,s0);
	let k00 = key_r[0].scale(&mut mac,s00);
	let k01 = key_r[1].scale(&mut mac,s01);
	let k10 = key_r[2].scale(&mut mac,s10);
	let k11 = key_r[3].scale(&mut mac,s11);
	let k0 = k00.or(&mut mac,&k01);
	let k1 = k10.or(&mut mac,&k11);
	let k = k0.or(&mut mac,&k1);
	let (s,_) = sum_r.add(&mut mac,&k,zero);
	// let s = sum.wrapping_add(key[(sum & 3) as usize]);
	// let s_r = Register::input(&mut mac,32);
	// constraints.append(&mut s_r.constraints(s as u64));

	let t3 = t2.xor(&mut mac,&s);
	let (v0_r_bis,_) = v0_r.add(&mut mac,&t3,zero);
	v0_r = v0_r_bis;

	let (sum_r_next,_) = sum_r.add(&mut mac,&delta_r,zero);
	sum_r = sum_r_next;

	let v0s4 = v0_r.shift_left(4,zero);
	let v0s5 = v0_r.shift_right(5,zero);
	let t1 = v0s4.xor(&mut mac,&v0s5);
	let (t2,_) = t1.add(&mut mac,&v0_r,zero);

	let s0 = sum_r.bit(11);
	let s1 = sum_r.bit(12);
	let s00 = mac.and(mac.not(s1),mac.not(s0));
	let s01 = mac.and(mac.not(s1),s0);
	let s10 = mac.and(s1,mac.not(s0));
	let s11 = mac.and(s1,s0);
	let k00 = key_r[0].scale(&mut mac,s00);
	let k01 = key_r[1].scale(&mut mac,s01);
	let k10 = key_r[2].scale(&mut mac,s10);
	let k11 = key_r[3].scale(&mut mac,s11);
	let k0 = k00.or(&mut mac,&k01);
	let k1 = k10.or(&mut mac,&k11);
	let k = k0.or(&mut mac,&k1);
	let (s,_) = sum_r.add(&mut mac,&k,zero);

	// let s = sum.wrapping_add(key[((sum >> 11) & 3) as usize]);
	// let s_r = Register::input(&mut mac,32);
	// constraints.append(&mut s_r.constraints(s as u64));
	let t3 = t2.xor(&mut mac,&s);
	let (v1_r_bis,_) = v1_r.add(&mut mac,&t3,zero);
	v1_r = v1_r_bis;

	if r + 1 == NROUND {
	    let d0 = v0_r.xor(&mut mac,&y0_r);
	    let d1 = v1_r.xor(&mut mac,&y1_r);
	    let d0 = d0.not(&mut mac);
	    let d1 = d1.not(&mut mac);
	    let d0 = d0.all_ones(&mut mac);
	    let d1 = d1.all_ones(&mut mac);
	    let cmp = mac.and(d0,d1);
	    out_constraints.push((cmp,true));
	}
    }

    out_constraints.append(&mut constraints.clone());
    mac.save_cnf("mac.cnf",&out_constraints).unwrap();
    mac.dump("mac.gt").unwrap();

    Register::dump(&mac,"mac.reg",
		   vec![
		       ("x0",&x0_r),
		       ("x1",&x1_r),
		       ("y0",&y0_r),
		       ("y1",&y1_r),
		       ("k0",&key_r[0]),
		       ("k1",&key_r[1]),
		       ("k2",&key_r[2]),
		       ("k3",&key_r[3]),
		       ("addr",&addr_r),
		       ("demux",&demux_r)
		   ]).unwrap();

    // for k in 0..4 {
    // 	constraints.append(&mut key_r[k].constraints(key[k] as u64));
    // }

    println!("Evaluating...");

    // if false {
    // 	let sz = bracket::SizeMorphism::new();
    // 	let s = mac.eval_morphism(&constraints,&sz);
    // 	for i in 0..s.len() {
    // 	    println!("{:05} {:5.1}",i,s[i].log2());
    // 	}
    // }

    // if false {
    // 	let trimo = bracket::TrimmedMorphism::new(5);
    // 	let trimos = mac.eval_morphism(&out_constraints,&trimo);
    // }

    // let inp = bracket::InputSetMorphism::new();
    // let inps = mac.eval_morphism(&out_constraints,&inp);

    // let v = mac.eval(&constraints);

    // for k in 0..4 {
    // 	println!("k{} {:08X} {:08X}",k,key[k],key_r[k].value(&v));
    // }

    // for i in 0..n {
    // 	let ti = &traffic[i];
    // 	let tri = &traffic_r[i];
    // 	println!("TR{} X:({:08X},{:08X})->({:08X},{:08X}) Y:({:08X},{:08X})->({:08X},{:08X})",
    // 		 i,
    // 		 ti.x.0,ti.x.1,
    // 		 tri.x.0.value(&v),tri.x.1.value(&v),
    // 		 ti.y.0,ti.y.1,
    // 		 tri.y.0.value(&v),tri.y.1.value(&v));
    // 	// for j in 0..32 {
    // 	//     let b0 = tri.y.0.bit(j);
    // 	//     let b1 = tri.y.1.bit(j);
    // 	//     println!("Y0[{:02}] : {:0128b}",j,inps[b0 as usize]);
    // 	//     println!("Y1[{:02}] : {:0128b}",j,inps[b1 as usize]);
    // 	//     // trimo.dump(&trimos[b0 as usize]);
    // 	// }
    // }
}
