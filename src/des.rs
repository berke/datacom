// Lower 56 bits of k

mod machine;

const SHIFT_SCHEDULE : u16 = 0b0011111101111110;

const PC2 : [u8;48] = [
    14,17,11,24,1,5,
    3,28,15,6,21,10,
    23,19,12,4,26,8,
    16,7,27,20,13,2,
    41,52,31,37,47,55,
    30,40,51,45,33,48,
    44,49,39,56,34,53,
    46,42,50,36,29,32
];

fn pc2(cd:u64)->u64 {
    let mut k = 0_u64;
    for i in 0..48 {
	k <<= 1;
	if cd & (1 << (PC2[47 - i] - 1)) != 0 {
	    k |= 1;
	}
    }
    k
}

fn key_schedule(k:u64)->[u64;16] {
    let mut c = (k & 0xfffffff) as u32;
    let mut d = ((k >> 28) & 0xfffffff) as u32;
    let mut ks = [0_u64;16];
    let mut sh = SHIFT_SCHEDULE;
    for i in 0..16 {
	if sh & 0x8000 != 0 {
	    c = c.rotate_left(2);
	    d = d.rotate_left(2);
	} else {
	    c = c.rotate_left(1);
	    d = d.rotate_left(1);
	}
	sh <<= 1;
	ks[i] = pc2(((c as u64) << 28) | (d as u64));
    }
    ks
}

const EXPANSION : [u8;48] = [
    32,1,2,3,4,5,
    4,5,6,7,8,9,
    8,9,10,11,12,13,
    12,13,14,15,16,17,
    16,17,18,19,20,21,
    20,21,22,23,24,25,
    24,25,26,27,28,29,
    28,29,30,31,32,1
];

fn expand(r:u32)->u64 {
    let mut e : u64 = 0;
    for i in 0..48 {
	e <<= 1;
	if r & (1 << (EXPANSION[47 - i] - 1)) != 0 {
	    e |= 1;
	}
    }
    e
}

const PERMUTATION : [u8;32] = [
    16,7,20,21,
    29,12,28,17,
    1,15,23,26,
    5,18,31,10,
    2,8,24,14,
    32,27,3,9,
    19,13,30,6,
    22,1,4,25
];

fn permute(x:u32)->u32 {
    let mut y : u32 = 0;
    for i in 0..32 {
	y <<= 1;
	if x & (1 << (PERMUTATION[31 - i] - 1)) != 0 {
	    y |= 1;
	}
    }
    y
}

const SBOXES : [u8;512] = [
    14,4,13,1,2,15,11,8,3,10,6,12,5,9,0,7,
    0,15,7,4,14,2,13,1,10,6,12,11,9,5,3,8,
    4,1,14,8,13,6,2,11,15,12,9,7,3,10,5,0,
    15,12,8,2,4,9,1,7,5,11,3,14,10,0,6,13,

    15,1,8,14,6,11,3,4,9,7,2,13,12,0,5,10,
    3,13,4,7,15,2,8,14,12,0,1,10,6,9,11,5,
    0,14,7,11,10,4,13,1,5,8,12,6,9,3,2,15,
    13,8,10,1,3,15,4,2,11,6,7,12,0,5,14,9,

    10,0,9,14,6,3,15,5,1,13,12,7,11,4,2,8,
    13,7,0,9,3,4,6,10,2,8,5,14,12,11,15,1,
    13,6,4,9,8,15,3,0,11,1,2,12,5,10,14,7,
    1,10,13,0,6,9,8,7,4,15,14,3,11,5,2,12,

    7,13,14,3,0,6,9,10,1,2,8,5,11,12,4,15,
    13,8,11,5,6,15,0,3,4,7,2,12,1,10,14,9,
    10,6,9,0,12,11,7,13,15,1,3,14,5,2,8,4,
    3,15,0,6,10,1,13,8,9,4,5,11,12,7,2,14,

    2,12,4,1,7,10,11,6,8,5,3,15,13,0,14,9,
    14,11,2,12,4,7,13,1,5,0,15,10,3,9,8,6,
    4,2,1,11,10,13,7,8,15,9,12,5,6,3,0,14,
    11,8,12,7,1,14,2,13,6,15,0,9,10,4,5,3,

    12,1,10,15,9,2,6,8,0,13,3,4,14,7,5,11,
    10,15,4,2,7,12,9,5,6,1,13,14,0,11,3,8,
    9,14,15,5,2,8,12,3,7,0,4,10,1,13,11,6,
    4,3,2,12,9,5,15,10,11,14,1,7,6,0,8,13,

    4,11,2,14,15,0,8,13,3,12,9,7,5,10,6,1,
    13,0,11,7,4,9,1,10,14,3,5,12,2,15,8,6,
    1,4,11,13,12,3,7,14,10,15,6,8,0,5,9,2,
    6,11,13,8,1,4,10,7,9,5,0,15,14,2,3,12,

    13,2,8,4,6,15,11,1,10,9,3,14,5,0,12,7,
    1,15,13,8,10,3,7,4,12,5,6,11,0,14,9,2,
    7,11,4,1,9,12,14,2,0,6,10,13,15,3,5,8,
    2,1,14,7,4,10,8,13,15,12,9,0,3,5,6,11,
];

fn substitute(e:u64)->u32 {
    let mut s : u32 = 0;
    let mut e0 = e;
    for i in 0..8 {
	s <<= 4;
	s |= SBOXES[(64*i + (e0 & 63)) as usize] as u32;
	e0 >>= 6;
    }
    s
}

fn f(r:u32,k:u64)->u32 {
    let x = expand(r) ^ k;
    permute(substitute(x))
}

fn sp(x:u64)->(u32,u32) {
    ((x >> 32) as u32, (x & 0xffffffff) as u32)
}

fn jn(x:u32,y:u32)->u64 {
    ((x as u64) << 32) | (y as u64)
}

fn enc(x:u64,ks:&[u64;16])->u64 {
    let mut lr = sp(x);
    for i in 0..16 {
	let (l,r) = lr;
	let l_next = r;
	let r_next = l^f(r,ks[i]);
	lr = (l_next,r_next);
    }
    jn(lr.0,lr.1)
}

fn main1() {
    let k : u64 = 0x123456789abcde;
    let ks = key_schedule(k);
    for i in 0..16 {
	println!("{:02} {:012X}",i,ks[i]);
    }

    let mut y_prev = 0;
    for x0 in 0_u64.. {
	let x = x0 ^ (x0 >> 1);
	let y = enc(x,&ks);
	println!("{:016X} -> {:016X} : {:016X}",x,y,y ^ y_prev);
	y_prev = y;
    }
}

use machine::{Gate,Op,Machine,Index};

fn rotate_left(v:&Vec<Index>,s:usize)->Vec<Index> {
    let n = v.len();
    (0..n).map(|k| v[(k + s) % n]).collect()
}

fn main() {
    let mut mac = Machine::new();
    let key : Vec<Index> = (0..56).map(|k| mac.get(&Gate::Input(k as u16))).collect();

    let mut c = Vec::from(&key[0..28]);
    let mut d = Vec::from(&key[28..56]);
    let mut sh = SHIFT_SCHEDULE;
    let mut ks = Vec::new();

    for i in 0..16 {
	if sh & 0x8000 != 0 {
	    c = rotate_left(&c,2);
	    d = rotate_left(&d,2);
	} else {
	    c = rotate_left(&c,1);
	    d = rotate_left(&d,1);
	}
	sh <<= 1;
	let mut ki = Vec::new();
	for j in 0..48 {
	    let k = (PC2[j] - 1) as usize;
	    if k < 28 {
		ki.push(c[k]);
	    } else {
		ki.push(d[k - 28]);
	    }
	}
	ks.push(ki);
    }

    let input : Vec<Index> = (0..64).map(|k| mac.get(&Gate::Input(k as u16))).collect();

    for i in 0..48 {
	let _ = mac.get(&Gate::Binop(Op::Xor,ks[0][i],input[i]));
    }

    // let kr1 = rotate_left(&key,11);
    mac.dump();
}

// fn key_schedule(k:u64)->[u64;16] {
//     let mut c = (k & 0xfffffff) as u32;
//     let mut d = ((k >> 28) & 0xfffffff) as u32;
//     let mut ks = [0_u64;16];
//     let mut sh = SHIFT_SCHEDULE;
//     for i in 0..16 {
// 	if sh & 0x8000 != 0 {
// 	    c = c.rotate_left(2);
// 	    d = d.rotate_left(2);
// 	} else {
// 	    c = c.rotate_left(1);
// 	    d = d.rotate_left(1);
// 	}
// 	sh <<= 1;
// 	ks[i] = pc2(((c as u64) << 28) | (d as u64));
//     }
//     ks
// }


// 6 -> 4
// notice that each line is a permutation
// so for each S-box
// the first two bits of the entry pick a permutation
// and then we apply a permutation over 4-bit words (16)
// -- about 2^44.25 permutations
//  -- decode 4-bit into 16
//  -- permute
//  -- encode

// decoder circuit complexity?
//   recursive
//     decoder(b[0..n]) =
//       ( b[0] & decoder(b[1..n] ,
//        !b[0] & decoder(b[1..n] )
//     prolly 2^n ish?
//     n = 4 so OK
//   IP

// n log n ?  4 * 16 ~ 64
// plus a few bits for selection -- good
// 56 gates avg. by Kwan
// prev. best: 61


// to instantiate OP(X,Y)
//   -- see if it already exists
//   -- see if OP(Y,X) exists if OP is commut.
//   -- see if !OP(X,Y) exists
//   -- more generally, for every existing term, pair of terms, etc.
//      compute the extra amount of gates to be added on top
//      to get what we want
//         ...


// 6 -> 4
//   let a = [010101 ... ]
//   let b = [00110011. ...]
//   let c = [00001111... ]
//   let d = [00000000 ]
//   let e = [000000000 ]
//
// focus on single bit

// 6 -> 1
// 64 possible functions
//   should be able to exhaustively enumerate all circuits with 6 inputs??
//
//     conject: not more than ~ 64 x 6 = 384 circuits
// 6 -> 2
//     the subcircuits will be in the table
//       only subcircuits that give correct answer for 1
//       plus those that give correct answer for 2
//       pick two circuits, fuse them - select smallest
// 6 -> 3
//      rinse and repeat
