mod xorwow;
mod utils;
mod tea;

use std::io::Write;
use xorwow::Xorwow;
use utils::*;

struct BinomialTester {
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
	    stats:[0;65],
	    count:0
	}
    }

    pub fn test<F:Fn(u64)->u64>(&mut self,count:usize,f:F,x0:u64) {
	let mut x = x0;
	for _ in 0..count {
	    let w = x.weight();
	    self.stats[w] += 1;
	    x = f(x);
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
		let z = (log2_n_actual.exp2() - log2_n_expected.exp2()).abs().log2() - log2_sd_n_expected;
		
		if z < -3.0 {
		    writeln!(fmt,"{:2} {:10.6} {:10.6} {:10.6}",
			     w,
			     log2_n_actual,
			     log2_n_expected,
			     z);
		}
	    }
	}
    }
}

fn main() {
    let mut xw = Xorwow::new(1234567);
    let mut bt = BinomialTester::new();
    let k = xw.gen_u128();
    let nround = 64;
    let count = 100000000;
    let x0 = xw.gen_u64();
    let f = |x| tea::encipher1(x,k,nround);
    bt.test(count,f,x0);
    bt.dump(&mut std::io::stdout());
}
