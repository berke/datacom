use std::cmp::Ordering;

mod cmath {
    use libc::c_double;

    #[link_name = "m"]
    extern "C" {
	pub fn erf(n: c_double) -> c_double;
    }
}

pub fn erf(x: f64) -> f64 {
    unsafe { cmath::erf(x) }
}

const N : usize = 16;

struct Xorwow {
    a:u32,
    b:u32,
    c:u32,
    d:u32,
    counter:u32
}

impl Xorwow {
    pub fn new(seed:u32)->Self {
	Xorwow{ a:seed,b:1,c:1,d:1,counter:0 }
    }
    pub fn reset(&mut self,seed:u32) {
	self.a = seed;
	self.b = 1;
	self.c = 1;
	self.d = 1;
	self.counter = 0;
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
	self.counter = self.counter.wrapping_add(362437);
	let r = t.wrapping_add(self.counter);
	r
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

fn main1() {
    let seed = 99123411;
    let mut xw = Xorwow::new(seed);
    let mut k1 = [0_u32;4];
    let mut k2 = [0_u32;4];

    k1 = [0xdeadbe33, 0x0badcafe, 0x12345678, 0x9abcdef0];

    const B : usize = 64;
    const M : usize = N * B;
    const K : usize = 1 << 30;
    const Q : usize = 10000;
    const RN : usize = N;
    
    let mut x0 = 0;
    let mut y0 = 0;
    let mut tr1 = [0_u64;N];
    let mut tri1 = [0_u64;N];
    let mut tr = [0_u64;N];
    let mut tri = [0_u64;N];
    // let mut sums = Vec::new();
    //sums.resize(K,[[[[0_i64;2];2];M];M]);
    //sums.resize(K,[0_i64;2*2*RN*B*RN*B]);

    let mut v = Vec::new();//[0.0_f64;K];
    let mut d_hist = [0_usize;N*B];
    let mut s = [0_i64;2*2*RN*B*RN*B];
    for k0 in 0..K {
	for i in 0..s.len() {
	    s[i] = 0;
	}
	// xw.reset(seed);
	//let k = (((k0 as u32) ^ k1[0]) & (K as u32 - 1)) as usize;
	let k = (
	    (if k0 == 0 { k1[0] } else { xw.next() }) & (K as u32 - 1)) as usize;
	// let k = k0;
	// println!("K {:02X}",k);
	// let ks = [0x88,0x20,0x77,0xff,0x21,0x30,0xcc,0xa0,0x80];
	// for k0 in 0..ks.len() {
	// let k = ks[k0];
	// let k = k0 ^ (k1[0] & 0xff) as usize;
	k2[0] =
	    // ((  (  xw.next() & 0xff000000
	    // 	| k1[1]      & 0x00ffffff) ) &
	    (xw.next() & !(K as u32 - 1))
	    // | (k1[0]     & 0xfffff000)
	    | (k as u32); // (xw.next() << 8) | (k as u32);
	k2[1] = xw.next();
	k2[2] = xw.next();
	k2[3] = xw.next();
	// k2[1] = k1[1];
	// k2[2] = k1[2];
	// k2[3] =
	//       (xw.next() & 0x00000000)
	//     | (k1[3]     & 0xffffffff);

	x0 = 0;
	y0 = 0;
	let d = hd(&k1,&k2);
	d_hist[d as usize] += 1;
	let mut std = 0;
	for _ in 0..Q {
	    x0 = xw.next();
	    y0 = xw.next();
	    let xy = (x0,y0);
	    let ab = h(xy,k1);
	    // let _ = htr(xy,k1,&mut tr1);
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
	    // let s = &mut sums[k];
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
	    // y0 = y0.wrapping_add(1);
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
	let mut se = 0.0;
	let mut v_tot = 0.0;
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
		let w = 1.0-(f(0)+f(1)+f(2)+f(3));
		//if w > 0.60 {
		{
		    // println!("{} {} {}",i1,i2,w);
		    v_tot += w;
		}
		j += 4;
	    }
	}
	v_tot /= (M*M) as f64;
	v.push(v_tot);
	println!("{:02X} {:12.7e} {} {:7.5}",k,v_tot,if k as u32 == k1[0]&255 {"*"} else {" "},se/(4*M*M) as f64);
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

struct YW {
    w:usize,
    n:usize,
    n0:usize,
    n0_set:bool,
    m:usize,
    x0:f64,
    dx:f64,
    h:Vec<usize>
}

impl YW {
    pub fn new(x0:f64,x1:f64,nx:usize)->Self {
	let dx = (x1 - x0) / (nx - 1) as f64;
	let mut h = Vec::new();
	h.resize(nx,0_usize);
	YW{ w:0, n:0, m:0, n0:0, n0_set:false, x0, dx, h }
    }
    pub fn add(&mut self,x:u32,n:u32) {
	self.w += x.count_ones() as usize;
	self.n += n as usize;
    }
    pub fn nccum(n:f64,x:f64)->f64 {
	(erf(n.ln().ln().sqrt()*x)+1.0)/2.0
    }
    pub fn dump(&self) {
	let mut c0 = 0.0;
	let mut e_tot = 0.0;
	let n = self.n0 as f64;
	for i in 0..self.h.len() {
	    let x = self.x0 + self.dx*i as f64;
	    let c = Self::nccum(n,x);
	    let dc = c - c0;
	    let p = self.h[i] as f64/self.m as f64;
	    let e = (p-dc).abs();
	    e_tot += p; // e;
	    println!("{:8.3} {:15.07e} {:15.07e} {:15.07e}",x,p,dc,e);
	    c0 = c;
	}
	e_tot *= self.dx;
	println!("E_TOT: {:15.07e}",e_tot);
    }
    pub fn next(&mut self) {
	if self.n0_set {
	    if self.n != self.n0 {
		panic!("Mismatch")
	    }
	} else {
	    self.n0 = self.n;
	    self.n0_set = true;
	}
	let n = self.n as f64;
	let x = (2.0 * self.w as f64 - n) / ((2.0*n*n.ln().ln()).sqrt());
	let j = (((x - self.x0) / self.dx).floor().max(0.0) as usize).min(self.h.len() - 1);
	self.h[j] += 1;
	self.m += 1;
	self.n = 0;
	self.w = 0;
    }
}

struct Clcg {
    a:u32,
    b:u32,
    q:u32
}

impl Clcg {
    pub fn new(a:u32,b:u32,seed:u32)->Self {
	Clcg{ a,b,q:seed }
    }
    // a = 0x343fd
    // b = 0x269ec3
    pub fn next(&mut self)->(u32,u32) {
	let q = self.a.wrapping_mul(self.q).wrapping_add(self.b);
	self.q = q;
	(0 | (q >> 16) & 32767, 15)
    }
}

// fn napprox(n:usize,x:f64)->f64 {
//     // Phi(x*sqrt(2*ln(ln(n))))
//     let n = n as f64;
//     ((x * n.ln().ln().sqrt()).erf() + 1.0) / 2.0
// }

fn main() {
    let m = std::env::args().nth(1).unwrap().parse::<usize>().unwrap();
    let n_shift = std::env::args().nth(2).unwrap().parse::<usize>().unwrap();
    //let mut yw = YW::new(-3.5,3.5,50);
    let mut yw = YW::new(-2.5,2.5,10);
    let seed = 99123411;
    let mut seeder = Xorwow::new(seed);
    // let m = 100;
    // let mut ss = Vec::new();
    let n_max = 1_usize << n_shift;
    if false {
	let k1 = [0xdeadbe33, 0x0badcafe, 0x12345678, 0x9abcdef0];
	for k in 0..m {
	    let seed = seeder.next();
	    // let mut xw = Clcg::new(seed);
	    //let mut xw = Xorwow::new(seed);
	    let mut xy = (seed,seed);
	    let mut n = 0_usize;
	    loop {
		// let (x,p) = xw.next();
		// let (x,p) = (xw.next(),32);
		xy = h(xy,k1);
		let (x,p) = (xy.0,32);
		yw.add(x,p);
		n += p as usize;
		if n >= n_max { break; }

		let (x,p) = (xy.1,32);
		yw.add(x,p);
		n += p as usize;
		if n >= n_max { break; }
	    }
	    yw.next();
	    // let s = yw.get();
	    // println!("{:22.14e}", s);
	    // if (k % 10) == 0 { eprintln!("{:05} {:22.14e} N={}", k, s, n); }
	}
    } else {
	for k in 0..m {
	    let seed = seeder.next();
	    let mut n = 0_usize;
	    // let mut st = Clcg::new(0x343fd,0x269ec3,seed);
	    // a = 0x343fd
	    // b = 0x269ec3
	    let mut st = Xorwow::new(seed);
	    // let st = rdrand::RdRand::new().unwrap();
	    loop {
		// let (x,p) = (st.try_next_u32().unwrap(),32);
		let (x,p) = (st.next(),32);
		// let (x,p) = st.next();
		yw.add(x,p);
		n += p as usize;
		if n >= n_max { break; }
	    }
	    yw.next();
	}
    }
    yw.dump();
}

// xw                   1000 26 3.569e-2
// xw                   2000 26 3.958e-2
// xw                    100 30 5.710e-2
// cl                   1000 26 4.978e-2
// cl                   2000 26 5.235e-2
// cl(1,12345)          2000 26 1.645e-1
// cl(0x343fd,1)        2000 26 4.873e-2
// cl(0x343fd,1)         100 30 1.049e-1
// cl(0x343fd,0x269ec3)  100 30 7.908e-2
// rdrand                100 26 5.708e-2
// rdrand                100 29 6.081e-2 !!
// rdrand               1000 24 4.182e-2 !!
