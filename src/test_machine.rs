mod machine;

use machine::{Gate,Op,Machine,Index,Register};

fn main() {
    let mut mac = Machine::new();
    let mut key1 = Register::input(&mut mac,0,32);
    let mut key2 = Register::input(&mut mac,32,32);
    let mut x = Register::input(&mut mac,64,32);
    let zero = mac.zero();
    for i in 0..4 {
	let y = key1.xor(&mut mac,&x);
	let (z,_c) = x.add(&mut mac,&key2,zero);
	x = z;
	key1 = key1.rotate_left(11);
	key2 = key2.rotate_left(7);
    }
    mac.dump();
}
