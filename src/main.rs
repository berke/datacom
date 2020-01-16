use std::io::{Read,Write};
use std::cmp::Ordering;

mod xtea;

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

fn jn2((x,y):(u32,u32))->u64 {
    jn(x,y)
}

fn jn(x:u32,y:u32)->u64 {
    ((x as u64) << 32) | (y as u64)
}

// const M : u64 = 1 << 32;
// const M : u64 = 265371653;
 const M : u64 = 75853;
// const E : u64 = 3;
const E : u64 = 17;

fn tbl()->[u32;31] {
    let mut eks = [0_u32;31];
    let mut ek = E;
    for k in 0..31 {
        eks[k] = ek as u32;
        ek = (ek * ek) % M;
    }
    eks
}

fn f(eks:[u32;31],x:u32)->u32 {
    let mut q : u64 = 1;
    let mut y = x;
    for k in 0..31 {
        if y == 0 {
            break;
        }
        if y & 1 != 0 {
            q = (q * eks[k] as u64) % M;
        }
        y >>= 1;
    }
    (q & 0xffffffff) as u32
}

fn f0(x:u32)->u32 {
    let mut q : u64 = 1;
    let mut y = x;
    let mut ek = E;
    for k in 0..31 {
        if y == 0 {
            break;
        }
        if y & 1 != 0 {
            q = (q * ek) % M;
        }
        y >>= 1;
        ek = (ek * ek) % M;
    }
    (q & 0xffffffff) as u32
}

fn f0_xtea(x:u32)->u32 {
    xtea::encipher((0xdeadbeef,0x0badcafe),[x,x,x,x]).0 & (M - 1) as u32
}

fn check_period() {
    let eks = tbl();
    let mut seen = Vec::new();
    seen.resize(((M + 63) >> 6) as usize,0_u64);
    for x in 0..M as u32 {
        if x & 0xffffff == 0 {
            println!("{:5.1}%",100.0*(x as f64)/(0x7fffffff as f64));
        }
        //let y = f(eks,x);
        let y = f0(x);
	if y >= M as u32 {
	    panic!("f({})={} > {}",x,y,M);
	}
        let i = (y >> 6) as usize;
        let si = seen[i];
        let j = y & 63;
        // println!("{:08X} {:08X}",x,y);
        if (si >> j) & 1 != 0 {
            panic!("SEEN {:08X} {}",y,x);
        }
        seen[i] = si | (1 << j);
    }
}

struct Table {
    t:usize,
    v:Vec<(u32,u32)>
}

impl Table {
    fn new(m:usize)->Self {
        let mut xw = Xorwow::new(1234567);
        let t = ((M as usize / m) as f64).sqrt().floor() as usize;
	let t = t * t;
        println!("M={} m={} t={}",M,m,t);
	let mut v = Self::fill(&mut xw,t,m);
	println!("Sorting...");
	v.sort_by(|(xa,ya),(xb,yb)| ya.cmp(yb));
	println!("Done");
        Table{
	    t,
            v
        }
    }

    fn fill(xw:&mut Xorwow,t:usize,m:usize)->Vec<(u32,u32)> {
	let mut v = Vec::new();
	let mut found = false;
	let mut seen = Vec::new();
	seen.resize(((M + 63) >> 6) as usize,0_u64);
	let mut coverage = Vec::new();
	coverage.resize(((M + 63) >> 6) as usize,0_u64);
	let mut ctr = 0;
	let mut k0;
	let mut dupes = 0_usize;
	for i in 0..m {
	    if i & 0xffff == 0 {
		println!("{}/{}",i,m);
	    }
	    // let mut k0 : u32 = (xw.next() ^ i as u32) % M as u32;
	    loop {
		let kk = jn2(xtea::encipher(sp(ctr),[0x12345678,0x9abcdef0,0x31415926,0x53581414]));
		// let kk = ctr;
		ctr += 1;
		k0 = (kk % M) as u32;
		if (coverage[(k0 >> 6) as usize] >> (k0 & 63)) & 1 == 0 {
		    break;
		}
		dupes += 1;
	    }
	    seen[(k0 >> 6) as usize] |= 1 << (k0 & 63);
	    coverage[(k0 >> 6) as usize] |= 1 << (k0 & 63);
	    let mut k = k0;
	    for j in 0..t {
		k = f0(k);
		// if (coverage[(k >> 6) as usize] >> (k & 63)) & 1 != 0 {
		//     println!("CYC {}",j);
		// }
		coverage[(k >> 6) as usize] |= 1 << (k & 63);
	    }
	    v.push((k0,k));
	}
	println!("Dupes: {}",dupes);
	let cov = coverage.iter().fold(0,|q,x| q + x.count_ones());
	println!("Coverage: {} ({:.3}%)",cov,100.0*cov as f64/M as f64);
	v
    }

    fn load(path:&str)->Self {
        let fd = std::fs::File::open(path).unwrap();
	let mut fd = std::io::BufReader::new(fd);
        let t = readu64(&mut fd) as usize;
        let m = readu64(&mut fd) as usize;
        let mut v = Vec::new();
	let mut y_last = 0;
        for i in 0..m {
	    let x = readu32(&mut fd);
	    let y = readu32(&mut fd);
	    if y < y_last {
		panic!("Incorrectly ordered table: y[{}]={} y[{}]={}",i-1,y_last,i,y);
	    }
	    v.push((x,y));
	    y_last = y;
	}
        Table{
            t,
	    v
        }
    }

    fn save(&self,path:&str) {
        let fd = std::fs::File::create(path).unwrap();
	let mut fd = std::io::BufWriter::new(fd);
        writeu64(&mut fd,self.t as u64);
        writeu64(&mut fd,self.v.len() as u64);
	for &(x,y) in self.v.iter() {
	    writeu32(&mut fd,x);
	    writeu32(&mut fd,y);
	}
    }

    fn search(&self,y:u32)->Option<u32> {
	let mut y0 = y;
	for t in 0..M as usize/self.t {
	    // println!("t={} y0={}",t,y0);
	    let v = &self.v;
	    match v.binary_search_by(|xy| xy.1.cmp(&y0)) {
		Err(_) => (),
		Ok(i) => {
		    let mut xi = v[i].0;
		    // println!("Found {}=f^{}({}) in table {}, index {} ({:?})!",y0,self.t,xi,t,i,v[i]);
		    for _ in 0..self.t {
			let yi = f0(xi);
			if yi == y {
			    println!("f({})={}",xi,y);
			    return Some(xi);
			}
			xi = yi;
		    }
		    // println!("No dice");
		}
	    }
	    y0 = f0(y0);
	}
	None
    }
}

fn writeu32<T:Write>(fd:&mut T,x:u32) {
    let mut a = [0_u8;4];
    a.copy_from_slice(&x.to_ne_bytes());
    fd.write_all(&a).unwrap();
}

fn writeu64<T:Write>(fd:&mut T,x:u64) {
    let mut a = [0_u8;8];
    a.copy_from_slice(&x.to_ne_bytes());
    fd.write_all(&a).unwrap();
}

fn readu32<T:Read>(fd:&mut T)->u32 {
    let mut a = [0_u8;4];
    fd.read_exact(&mut a).unwrap();
    u32::from_le_bytes(a)
}

fn readu64<T:Read>(fd:&mut T)->u64 {
    let mut a = [0_u8;8];
    fd.read_exact(&mut a).unwrap();
    u64::from_le_bytes(a)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = &args.next().unwrap();
    let targets = args.map(|x| x.parse::<u32>().unwrap()).collect::<Vec<u32>>();
    let tbl =
        if std::path::Path::new(path).exists() {
            println!("Loading existing table from {}",path);
            Table::load(path)
        } else {
            println!("Generating new table");
            let tbl = Table::new(20000);
            tbl.save(path);
            tbl
        };
    println!("Searching");
    for target in targets {
	println!("TARGET {}",target);
	match tbl.search(target) {
	    None => {
		println!("NOT FOUND");
		for k in 0..M as u32 {
		    if f0(k) == target {
			println!("SHOULD HAVE FOUND {}",k);
		    }
		}
	    }
	    Some(x) => {
		let y = f0(x);
		println!("{} => {}",x,y);
	    }
	}
    }
}
