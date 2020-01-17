mod xtea;
mod machine;
mod xorwow;

use xorwow::Xorwow;
use machine::{Gate,Op,Machine,Register};

struct Traffic {
    x:(u32,u32),
    y:(u32,u32)
}

const NROUND : usize = 2;

fn main() {
    let mut mac = Machine::new();
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
    let k0 = xw.next();
    let k1 = xw.next();
    let k2 = xw.next();
    let k3 = xw.next();
    let key = [k0,k1,k2,k3];

    // Generate
    let n = 1;
    let mut traffic = Vec::new();
    for _ in 0..n {
	let x0 = xw.next();
	let x1 = xw.next();
	let (y0,y1) = xtea::encipher((x0,x1),key,NROUND);
	traffic.push(Traffic{ x:(x0,x1),y:(y0,y1) });
	println!("{:08X} {:08X} -> {:08X} {:08X}",x0,x1,y0,y1);
    }

    let k0_r = Register::input(&mut mac,32);
    let k1_r = Register::input(&mut mac,32);
    let k2_r = Register::input(&mut mac,32);
    let k3_r = Register::input(&mut mac,32);
    let key_r = [k0_r,k1_r,k2_r,k3_r];

    let mut constraints = Vec::new();

    // let and = |x,y| mac.and(x,y);
    // let or = |x,y| mac.or(x,y);
    // let mut xor = |x:&Register,y:&Register| x.xor(&mut mac,&y);
    // let not = |x| mac.not(x);

    // Encode
    for i in 0..n {
	let Traffic{ x:(x0,x1),y:(y0,y1) } = traffic[i];
	let x0_r = Register::input(&mut mac,32);
	let x1_r = Register::input(&mut mac,32);
	// let y0_r = Register::input(&mut mac,32);
	// let y1_r = Register::input(&mut mac,32);
	constraints.append(&mut x0_r.constraints(x0 as u64));
	constraints.append(&mut x1_r.constraints(x1 as u64));
	let delta = 0x9e3779b9_u32;
	let mut sum : u32 = 0;
	let mut v0_r = x0_r;
	let mut v1_r = x1_r;
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

            let s = sum.wrapping_add(key[(sum & 3) as usize]);
	    let mut s_r = Register::input(&mut mac,32);
	    constraints.append(&mut s_r.constraints(s as u64));

	    let t3 = t2.xor(&mut mac,&s_r);
	    let (v0_r,_) = v0_r.add(&mut mac,&t3,zero);

	    sum += delta;

	    let v0s4 = v0_r.shift_left(4,zero);
	    let v0s5 = v0_r.shift_right(5,zero);
	    let t1 = v0s4.xor(&mut mac,&v0s5);
	    let (t2,_) = t1.add(&mut mac,&v0_r,zero);
            let s = sum.wrapping_add(key[((sum >> 11) & 3) as usize]);
	    let mut s_r = Register::input(&mut mac,32);
	    constraints.append(&mut s_r.constraints(s as u64));
	    let t3 = t2.xor(&mut mac,&s_r);
	    let (v1_r,_) = v1_r.add(&mut mac,&t3,zero);

	    if r + 1 == NROUND {
		constraints.append(&mut v0_r.constraints(y0 as u64));
		constraints.append(&mut v1_r.constraints(y1 as u64));
	    }
	}
    }


    // // let kc1 = key1.constraints(k1 as u64);

    // let a = Register::input(&mut mac,32);
    // let b = Register::input(&mut mac,32);
    // let zero = mac.zero();
    // let (c,_c) = a.add(&mut mac,&b,zero);
    // // let c = a.or(&mut mac,&b);
    // let a0 = xw.next();
    // let b0 = xw.next();
    // // let a0 = 0b00110011_u8;
    // // let b0 = 0b01011100_u8;
    // let c0 = a0.wrapping_add(b0);
    // // let c0 = a0 | b0;
    // let mut a1 = a.constraints(a0 as u64);
    // let mut c1 = c.constraints(c0 as u64);
    // a1.append(&mut c1);
    mac.save_cnf("mac.cnf",&constraints);
    // mac.dump();
    // println!("{:032b} + {:032b} = {:032b}",a0,b0,c0);

    for k in 0..4 {
	constraints.append(&mut key_r[k].constraints(key[k] as u64));
    }

    println!("Evaluating...");
    mac.eval(&constraints);
}
