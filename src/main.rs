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

fn main() {
    let mut xw = Xorwow::new();
    let mut k1 = [0_u32;4];
    let mut k2 = [0_u32;4];

    k1 = [0xdeadbe77, 0x0badcafe, 0x12345678, 0x9abcdef0];

    const B : usize = 64;
    const M : usize = N * B;
    const K : usize = 256;
    const Q : usize = 1000;
    const RN : usize = N;
    
    let mut x0 = 0;
    let mut y0 = 0;
    let mut tr1 = [0_u64;N];
    let mut tri1 = [0_u64;N];
    let mut tr = [0_u64;N];
    let mut tri = [0_u64;N];
    let mut sums = Vec::new();
    sums.resize(K,[[[[0_i64;2];2];M];M]);

    let mut v = [0.0_f64;K];
    for k in 0..K {
	k2[0] =
	      (xw.next() & 0x00000800)
	    | (k1[0]     & 0xfffff700)
	    | (k as u32); // (xw.next() << 8) | (k as u32);
	// k2[1] = xw.next();
	// k2[2] = xw.next();
	// k2[3] = xw.next();
	k2[1] = k1[1];
	k2[2] = k1[2];
	k2[3] = k1[3];

	x0 = 0;
	y0 = 0;
	for _ in 0..Q {
	    let xy = (x0,y0);
	    let ab = h(xy,k1);
	    // let _ = htr(xy,k1,&mut tr1);
	    // let _ = htri(ab,k1,&mut tri1);
	    // println!("XY={:08X}{:08X} AB={:08X}{:08X}",xy.0,xy.1,ab.0,ab.1);
	    // for r in 0..N { print!(" {:016X}",tr1[r]); } println!("");
	    // for r in 0..N { print!(" {:016X}",tri1[r]); } println!("");

	    htr(xy,k2,&mut tr);
	    htri(ab,k2,&mut tri);
	    // let mut s = 0;
		// s += ((tr1[r].0^tr2[r].0).count_ones() +
		//       (tr1[r].1^tr2[r].1).count_ones()) as u64;
	    for r1 in 0..RN {
		let t1 = tr[r1];
		for r2 in 0..RN {
		    let t2 = tri[r2];
		    for i1 in 0..B {
			let c1 = ((t1 >> i1) & 1) as usize;
			for i2 in 0..B {
			    let c2 = ((t2 >> i2) & 1) as usize;
			    sums[k][r1 * B + i1][r2 * B + i2][c1][c2] += 1;
			}
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
	for i1 in 0..M {
	    for i2 in 0..M {
		let s = &sums[k][i1][i2];
		let t = s[0][0] + s[0][1] + s[1][0] + s[1][1];
		let t = t as f64;
		let f = |i:usize,j:usize| {
		    let c = s[i][j];
		    if c == 0 {
			0.0
		    } else {
			let p = c as f64 / t;
			-p*p.log2()
		    }
		};
		v[k] += f(0,0)+f(0,1)+f(1,0)+f(1,1);
	    }
	}
	println!("{:02X} {} {}",k,v[k],if k as u32 == k1[0]&255 {"*"} else {""});
    }
}
