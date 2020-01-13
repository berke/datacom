use std::cmp::Ordering;

struct Xorwow {
    a:u32,
    b:u32,
    c:u32,
    d:u32,
    counter:u32
}

impl Xorwow {
    pub fn new(seed:u64)->Self {
        let (a,b) = sp(seed);
	Xorwow{ a,b,c:1,d:1,counter:0 }
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
	let s = self.a;
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
    pub fn next64(&mut self)->u64 {
        let a = self.next();
        let b = self.next();
        jn(a,b)
    }
}

fn sp(x:u64)->(u32,u32) {
    ((x >> 32) as u32, (x & 0xffffffff) as u32)
}

fn jn(x:u32,y:u32)->u64 {
    ((x as u64) << 32) | (y as u64)
}

fn h(x:u64,k:u64)->u64 {
    x.wrapping_add(k)
}

fn hi(x:u64,k:u64)->u64 {
    x.wrapping_sub(k)
}

pub fn lit(m:usize,w:usize)->f64 {
    let m = m as f64;
    (2.0 * w as f64 - m) / ((2.0*m*m.ln().ln()).sqrt())
}

fn dens(n:usize,x:f64)->f64 {
  //sqrt(log(log(n as f64))/pi)*exp(-x.^2*log(log(n)));
    let l = (n as f64).ln().ln();
    (-x*x*l).exp() * (l/std::f64::consts::PI).sqrt()
}

fn sample(gseed:u64,kseed:u64,k0:u64,m:usize,n:usize,his:&mut [usize],h0:usize,hshift:u32,hn:usize) {
    let mut his = &mut his[0..hn];
    let mut kxw = Xorwow::new(kseed);
    for j in 0..hn {
        his[j] = 0;
    }
    for i in 0..m {
        let mut gxw = Xorwow::new(gseed);
        let mut p : u64 = 0;
        let k = kxw.next64();
        for j in 0..n {
            let x = gxw.next64();
            let y = h(x,k0);
            let x2 = hi(y,k);
            p += x2.count_ones() as u64;
        }
        if p as usize >= h0 {
            let h = ((p as usize - h0) >> hshift).min(hn-1);
            his[h] += 1;
        }
        
        // let x = lit(n*64,p) as f64;
        // let p = dens(n*64,x);
        // sc += p*x/m as f64; // (x/m as f64 - d).abs();
    }
}

fn main() {
    let m = std::env::args().nth(1).unwrap().parse::<usize>().unwrap();
    let n = std::env::args().nth(2).unwrap().parse::<usize>().unwrap();
    let mut xw = Xorwow::new(1235);
    let k0 = xw.next64();
    let gseed = 92598;
    let kseed = 31284;
    let mut his = Vec::new();
    let hshift = 8;
    let hn = 64;
    let hshift = 8;
    let h0 = 32*n - (hn<<(hshift-1));
    his.resize(hn,0);
    sample(gseed,kseed,k0,m,n,&mut his,h0,hshift,hn);
    for j in 0..hn {
        println!("{} {}",h0+(j<<hshift),his[j]);
    }
}
