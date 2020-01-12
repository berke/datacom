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
	self.counter += 362437;
	let r = t + self.counter;
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

fn mix(b:usize,n:usize,x:u32,y:u32)->u32 {
    let mask = ((1 << n) - 1) << b;
    (x & mask) | (y & !mask)
}

fn main() {
    let seed = 99123411;
    let mut xw = Xorwow::new(seed);
    let mut xw2 = Xorwow::new(seed);
    let mut xw3 = Xorwow::new(seed);
    let mut k1 = [0_u32;4];
    let mut k2 = [0_u32;4];

    k1 = [0xde33adb2, 0x0badcafe, 0x12345678, 0x9abcdef0];

    const B : usize = 64;
    const M : usize = N * B;
    const LOG2K : usize = 8;
    const K : usize = 1 << LOG2K;
    const Q : usize = 100;
    const P : usize = 100;
    const RN : usize = N;
    
    let mut x0 = 0;
    let mut y0 = 0;
    let mut tr1 = [0_u64;N];
    let mut tri1 = [0_u64;N];
    let mut tr = [0_u64;N];
    let mut tri = [0_u64;N];

    let mut v = [0_i64;K];
    let mut d_hist = [0_usize;N*B];
    let mut d_min = 0;
    let mut s = [0_i64;2*2*RN*B*RN*B];
    let mut k_closest = [0_u32;4];
    let mut first = true;
    loop {
        for k0 in 0..K {
            // if (k0 & 255) == 0 || true { println!(" {}/{}",k0,K); }
            let k = k0;
            // x0 = 0;
            // y0 = 0;
            //xw.reset(seed);
            // let mut v_tot = 0.0;
            let mut v_min = [0_i64,0_i64];
            for p in 0..P {
                k2[0] = mix(0,LOG2K,k as u32,xw.next());
                // k2[0] = mix(0,LOG2K,k as u32,mix(LOG2K,8,xw.next(),k1[0]));
                k2[1] = k1[1];//xw.next();
                k2[2] = k1[2];//xw.next();
                k2[3] = mix(0,8,xw.next(),k1[3]);
                let d = hd(&k1,&k2);
                d_hist[d as usize] += 1;
                if first || d < d_min {
                    d_min = d;
                    k_closest = k2;
                }

                xw2.reset(seed);
                for q in 0..Q {
                    x0 = xw2.next();
                    y0 = xw2.next();
                    let xy = (x0,y0);
                    for r in 0..2 {
                        let ab =
                            if r == 0 {
                                h(xy,k1)
                            } else {
                                (xw3.next(),
                                 xw3.next())
                            };
                        // let _ = htr(xy,k1,&mut tr1);
                        // let _ = htri(ab,k1,&mut tri1);
                        // println!("XY={:08X}{:08X} AB={:08X}{:08X}",xy.0,xy.1,ab.0,ab.1);
                        // for r in 0..N { print!(" {:016X}",tr1[r]); } println!("");
                        // for r in 0..N { print!(" {:016X}",tri1[r]); } println!("");

                        htr(xy,k2,&mut tr);
                        htri(ab,k2,&mut tri);
                        let v = hd64(&tr,&tri);
                        //if q == 0 || v < v_min {
                        v_min[r] += v as i64;
                        //}
                        // x0 = x0.wrapping_add(123408211);
	                // y0 = y0.wrapping_add(585748839);
	            }
                }
            }
	    v[k] += (v_min[0] - v_min[1]); // .abs();
            first = false;
        }
        let mut col = 0;
        for i in 0..N*B {
            if d_hist[i] > 0 {
                print!("{:3} {:8}",i,d_hist[i]);
                col += 1;
                if col == 5 {
                    println!();
                    col = 0;
                }
	    }
        }
        if col > 0 {
            println!();
        }
        println!("K_closest: {:08X} {:08X} {:08X} {:08X}",
                 k_closest[0],
                 k_closest[1],
                 k_closest[2],
                 k_closest[3]);
        println!("     diff: {:08X} {:08X} {:08X} {:08X}",
                 k_closest[0]^k1[0],
                 k_closest[1]^k1[1],
                 k_closest[2]^k1[2],
                 k_closest[3]^k1[3]);
        let mut idx = (0..K).collect::<Vec<usize>>();
        idx.sort_by(|&i, &i|
                    if v[i] < v[j] {
                        Ordering::Less
                    } else if v[i] > v[j] {
                        Ordering::Greater
		    } else {
		        Ordering::Equal
		    }
        );
        for i in 0..(K - 1).min(25) {
            let k = idx[i];
            println!("EST {:3} {:04X} @{:3} {:12} {:12}", i, k,
                     ((k as u32 ^ k1[0]) & (K - 1) as u32).count_ones(),
                     v[k], v[idx[i+1]] - v[k]);
        }
    }
    // let mut v_min = v[0];
    // let mut k_min = 0;
    // for k in 1..K {
    // 	if v[k] < v_min {
    // 	    k_min = k;
    // 	    v_min = v[k];
    // 	}
    // }
    // println!("EST: {:02X} {}", k_min, v_min);
}
