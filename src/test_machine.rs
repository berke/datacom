mod machine;
mod xorwow;

use xorwow::Xorwow;
use machine::{Gate,Op,Machine,Index,Register};

fn main() {
    let mut mac = Machine::new();
    // let mut key1 = Register::input(&mut mac,0,32);
    // let mut key2 = Register::input(&mut mac,32,32);
    // let mut x = Register::input(&mut mac,64,32);
    // let zero = mac.zero();
    // for i in 0..4 {
    // 	let y = key1.xor(&mut mac,&x);
    // 	let (z,_c) = x.add(&mut mac,&key2,zero);
    // 	x = z;
    // 	key1 = key1.rotate_left(11);
    // 	key2 = key2.rotate_left(7);
    // }
    // mac.dump();
    let mut xw = Xorwow::new(129837471234567);
    // let k1 = xw.next();
    // let k2 = xw.next();
    // let kc1 = key1.constraints(k1 as u64);

    let a = Register::input(&mut mac,0,32);
    let b = Register::input(&mut mac,32,32);
    let zero = mac.zero();
    let (c,_c) = a.add(&mut mac,&b,zero);
    // let c = a.or(&mut mac,&b);
    let a0 = xw.next();
    let b0 = xw.next();
    // let a0 = 0b00110011_u8;
    // let b0 = 0b01011100_u8;
    let c0 = a0.wrapping_add(b0);
    // let c0 = a0 | b0;
    let mut a1 = a.constraints(a0 as u64);
    let mut c1 = c.constraints(c0 as u64);
    a1.append(&mut c1);
    mac.save_cnf("mac.cnf",&a1);
    mac.dump();
    println!("{:032b} + {:032b} = {:032b}",a0,b0,c0);
}
