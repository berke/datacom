mod xorwow;
mod utils;
mod tea;
mod xtea;

use std::io::Write;
use xorwow::Xorwow;
use utils::*;

struct BinomialTester {
    xw:Xorwow,
    stats:[usize;65],
    count:usize
}

fn range_prod(n1:usize,n2:usize)->usize {
    (n1..=n2).fold(1,|q,x| q*x)
}

fn log2_range_prod(n1:usize,n2:usize)->f64 {
    (n1..=n2).fold(0.0,|q,x| q + (x as f64).log2())
}

fn factorial(n:usize)->usize {
    range_prod(1,n)
}

fn log2_factorial(n:usize)->f64 {
    log2_range_prod(1,n)
}

fn binomial(out_of:usize,select:usize)->usize {
    range_prod(out_of-select+1,out_of)/factorial(select)
}

fn log2_binomial(out_of:usize,select:usize)->f64 {
    log2_range_prod(out_of-select+1,out_of) - log2_factorial(select)
}

impl BinomialTester {
    pub fn new()->Self {
	Self{
	    xw:Xorwow::new(1234),
	    stats:[0;65],
	    count:0
	}
    }

    pub fn test<F:Fn(u64)->u64>(&mut self,count:usize,f:F,x0:u64) {
	let mut x = x0;
	for _ in 0..count {
	    let y = f(x);
	    let d = 1_u64 << self.xw.integer(64);
	    //let xp = x.wrapping_sub(d);
	    let xp = x ^ d;
	    let yp = f(xp);
	    let w = (y ^ yp).weight();
	    self.stats[w] += 1;
	    x = y;
	}
	self.count += count;
    }

    pub fn dump<F:Write>(&self,fmt:&mut F) {
	let c = (self.count as f64).log2();
	for w in 0..65 {
	    let log2_n_actual = (self.stats[w] as f64).log2();
	    let log2_n_expected = log2_binomial(64,w) - 64.0 + c;
	    if log2_n_expected >= 0.0 {
		let log2_sd_n_expected = log2_n_expected / 2.0; // xxx
		let e = (log2_n_actual.exp2() - log2_n_expected.exp2()).abs();
		let z = (log2_n_actual.exp2() - log2_n_expected.exp2()).abs().log2() - log2_sd_n_expected;
		
		if z > 0.0 {
		    writeln!(fmt,"{:2} {:10.6} {:10.6} {:10.6} {:+10.1}/2^{:10.6}",
			     w,
			     log2_n_actual,
			     log2_n_expected,
			     z,
			     e,
			     log2_sd_n_expected).unwrap();
		}
	    }
	}
    }
}

fn main() {
    let mut xw = Xorwow::new(1234568);
    let mut bt = BinomialTester::new();
    let k = xw.gen_u128();
    let nround = 4;
    let passes = 1000;
    let count = 10000;
    let x0 = xw.gen_u64();
    let f1 = |x| tea::encipher1(x,k,nround);
    let f2 = |x| xtea::encipher1(x,k,nround);
    let f3 = |x:u64| {
	use blake2::{Blake2b,Digest};
	let mut h = Blake2b::new();
	let mut xb = [0;8];
	xb.copy_from_slice(&x.to_le_bytes());
	h.update(&xb);
	xb.copy_from_slice(&h.finalize()[0..8]);
	u64::from_le_bytes(xb)
    };
    for _ in 0..passes {
	let x = xw.gen_u64();
	bt.test(count,f2,x);
    }
    bt.dump(&mut std::io::stdout());
}
