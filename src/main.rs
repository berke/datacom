use std::cmp::Ordering;

const N : usize = 4;

struct Xorwow {
    a:u32,
    b:u32,
    c:u32,
    d:u32,
    counter:u32
}

impl Xorwow {
    pub fn new()->Self {
	Xorwow{ a:1,b:1,c:1,d:1,counter:0 }
    }
    pub fn next(&mut self)->u32 {
	let mut t = self.d;
	let mut s = self.a;
	self.d = self.c;
	self.c = self.b;
	self.b = s;
	t ^= t >> 2;
	t ^= t << 1;
	t ^= s ^ (s << 4);
	self.a = t;
	self.counter += 362437;
	t + self.counter
    }
}

fn f(x:u32,k0:u32,k1:u32)->u32 {
    x.rotate_left(11).wrapping_add(k0).rotate_left(20) ^ k1
}

fn g(xy:(u32,u32),k:[u32;4])->(u32,u32) {
    let mut x = xy.0;
    let mut y = xy.1;
    x ^= f(y,k[0],k[1]);
    y ^= f(x,k[1],k[2]);
    x ^= f(y,k[2],k[3]);
    y ^= f(x,k[3],k[0]);
    (x,y)
}

fn gi(xy:(u32,u32),k:[u32;4])->(u32,u32) {
    let mut x = xy.0;
    let mut y = xy.1;
    y ^= f(x,k[3],k[0]);
    x ^= f(y,k[2],k[3]);
    y ^= f(x,k[1],k[2]);
    x ^= f(y,k[0],k[1]);
    (x,y)
}

fn h(xy0:(u32,u32),k:[u32;4])->(u32,u32) {
    let mut xy = xy0;
    for _r in 0..N-1 {
	xy = g(xy,k);
    }
    xy
}

fn hi(xy0:(u32,u32),k:[u32;4])->(u32,u32) {
    let mut xy = xy0;
    for _r in 0..N-1 {
	xy = gi(xy,k);
    }
    xy
}

fn q(xy:(u32,u32))->u64 {
    ((xy.0 as u64) << 32) | xy.1 as u64
}

fn htr(xy0:(u32,u32),k:[u32;4],tr:&mut [u64;N]) {
    let mut xy = xy0;
    for r in 0..N-1 {
	tr[r] = q(xy);
	xy = g(xy,k);
    }
    tr[N-1] = q(xy);
}

fn htri(xy0:(u32,u32),k:[u32;4],tr:&mut [u64;N]) {
    let mut xy = xy0;
    for r in 0..N-1 {
	tr[N-1-r] = q(xy);
	xy = gi(xy,k);
    }
    tr[0] = q(xy);
}

fn hd(k1:&[u32],k2:&[u32])->u32 {
    let mut d = 0;
    let m = k1.len();
    for i in 0..m {
	d += (k1[i]^k2[i]).count_ones();
    }
    d
}

fn hd64(k1:&[u64],k2:&[u64])->u32 {
    let mut d = 0;
    let m = k1.len();
    for i in 0..m {
	d += (k1[i]^k2[i]).count_ones();
    }
    d
}

fn main() {
    let mut xw = Xorwow::new();
    let mut k1 = [0_u32;4];
    let mut k2 = [0_u32;4];

    k1 = [0xdeadbe55, 0x0badcafe, 0x12345678, 0x9abcdef0];

    const B : usize = 64;
    const M : usize = N * B;
    const K : usize = 16;
    const Q : usize = 100000;
    const RN : usize = 4;
    
    let mut x0 = 0;
    let mut y0 = 0;
    let mut tr1 = [0_u64;N];
    let mut tri1 = [0_u64;N];
    let mut tr = [0_u64;N];
    let mut tri = [0_u64;N];
    let mut sums = Vec::new();
    //sums.resize(K,[[[[0_i64;2];2];M];M]);
    sums.resize(K,[0_i64;2*2*RN*B*RN*B]);

    let mut v = [0.0_f64;K];
    let mut d_hist = [0_usize;N*B];
    for k0 in 0..K {
	let k = k0 ^ 0x5;
	// println!("K {:02X}",k);
	// let ks = [0x88,0x20,0x77,0xff,0x21,0x30,0xcc,0xa0,0x80];
	// for k0 in 0..ks.len() {
	// let k = ks[k0];
	// let k = k0 ^ (k1[0] & 0xff) as usize;
	k2[0] =
	    (xw.next() & 0xfffffff0)
	    // | (k1[0]     & 0xfffff000)
	    | (k as u32); // (xw.next() << 8) | (k as u32);
	k2[1] = xw.next();
	k2[2] = xw.next();
	k2[3] = xw.next();
	// k2[1] = k1[1];
	// k2[2] = k1[2];
	// k2[3] =
	//     (xw.next() & 0xfffff000)
	//     | (k1[3]     & 0x00000fff);

	x0 = 0;
	y0 = 0;
	let d = hd(&k1,&k2);
	d_hist[d as usize] += 1;
	let mut std = 0;
	for _ in 0..Q {
	    let xy = (x0,y0);
	    let ab = h(xy,k1);
	    let _ = htr(xy,k1,&mut tr1);
	    // let _ = htri(ab,k1,&mut tri1);
	    // println!("XY={:08X}{:08X} AB={:08X}{:08X}",xy.0,xy.1,ab.0,ab.1);
	    // for r in 0..N { print!(" {:016X}",tr1[r]); } println!("");
	    // for r in 0..N { print!(" {:016X}",tri1[r]); } println!("");

	    htr(xy,k2,&mut tr);
	    htri(ab,k2,&mut tri);
	    // let td = hd64(&tr1,&tr);
	    // std += td;
	    // let mut s = 0;
	    // s += ((tr1[r].0^tr2[r].0).count_ones() +
	    //       (tr1[r].1^tr2[r].1).count_ones()) as u64;
	    let mut j = 0;
	    let s = &mut sums[k];
	    for r1 in 0..RN {
		let t1 = tr[r1];
		for r2 in 0..RN {
		    let t2 = tri[r2];
		    let mut c1 = t1;
		    for i1 in 0..B {
			let mut c2 = t2;
			for i2 in 0..B {
			    s[(j+((c1&1)<<1)+(c2&1)) as usize] += 1;
			    c2 >>= 1;
			    j += 4;
			}
			c1 >>= 1;
		    }
		}
	    }
	    y0 = y0.wrapping_add(1);
	}
	// for i1 in 0..M {
	//     println!("{} {} {} {} {}",
	// 	     i1,
	// 	     sums[k][i1][i1][0][0],
	// 	     sums[k][i1][i1][0][1],
	// 	     sums[k][i1][i1][1][0],
	// 	     sums[k][i1][i1][1][1]);
	// }
	let mut j = 0;
	let s = &mut sums[k];
	let mut se = 0.0;
	for i1 in 0..M {
	    for i2 in 0..M {
		let t = s[j+0] + s[j+1] + s[j+2] + s[j+3];
		let t = t as f64;
		let mut f = |ij:usize| {
		    let c = s[j+ij];
		    if c == 0 {
			0.0
		    } else {
			let p = c as f64 / t;
			let e = -p*p.log2()-0.25;
			se += e;
			e
		    }
		};
		v[k] += f(0)+f(1)+f(2)+f(3);
		j += 4;
	    }
	}
	println!("{:02X} {:12.3} {} {:7.5}",k,v[k],if k as u32 == k1[0]&255 {"*"} else {" "},se/(4*M*M) as f64);
    }
    for i in 0..N*B {
	if d_hist[i] > 0 {
	    println!("{:3} {}",i,d_hist[i]);
	}
    }
    let mut idx = (0..K).collect::<Vec<usize>>();
    idx.sort_by(|&i, &j|
		if v[i] < v[j] {
		    Ordering::Less
		} else if v[i] > v[j] {
		    Ordering::Greater
		} else {
		    Ordering::Equal
		}
    );
    // let mut v_min = v[0];
    // let mut k_min = 0;
    // for k in 1..K {
    // 	if v[k] < v_min {
    // 	    k_min = k;
    // 	    v_min = v[k];
    // 	}
    // }
    // println!("EST: {:02X} {}", k_min, v_min);
    for i in 0..K.min(25) {
	let k = idx[i];
	println!("EST {:3} {:02X} {}", i, k, v[k]);
    }
}
