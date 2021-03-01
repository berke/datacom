use crate::xorwow::Xorwow;

pub fn jn128(x0:u32,x1:u32,x2:u32,x3:u32)->u128 {
    ((x0 as u128) << 96) |
    ((x1 as u128) << 64) |
    ((x2 as u128) << 32) |
    (x3 as u128)
}

pub fn jn(x0:u32,x1:u32)->u64 {
    ((x0 as u64) << 32) | x1 as u64
}

pub fn sp(x:u64)->(u32,u32) {
    ((x >> 32) as u32,(x & 0xffffffff) as u32)
}

pub fn sp128(x:u128)->[u32;4] {
    [(x >> 96) as u32,
     ((x >> 64) & 0xffffffff) as u32,
     ((x >> 32) & 0xffffffff) as u32,
     (x & 0xffffffff) as u32]
}

pub trait Hamming {
    fn weight(&self)->usize;
}

impl Hamming for u32 {
    fn weight(&self)->usize {
	self.count_ones() as usize
    }
}

impl Hamming for u64 {
    fn weight(&self)->usize {
	self.count_ones() as usize
    }
}

impl Hamming for u128 {
    fn weight(&self)->usize {
	self.count_ones() as usize
    }
}

pub trait Rng {
    fn uniform(&mut self)->f64;
    fn integer(&mut self,n:usize)->usize;
    fn gen_u64(&mut self)->u64;
    fn gen_u128(&mut self)->u128;
}

impl Rng for Xorwow {
    fn uniform(&mut self)->f64 {
	self.next() as f64 / ((1_u64 << 32) - 1) as f64
    }

    fn integer(&mut self,n:usize)->usize {
	((self.uniform() * n as f64).floor() as usize).min(n-1)
    }

    fn gen_u64(&mut self)->u64 {
	let x0 = self.next();
	let x1 = self.next();
	jn(x0,x1)
    }

    fn gen_u128(&mut self)->u128 {
	let x0 = self.next();
	let x1 = self.next();
	let x2 = self.next();
	let x3 = self.next();
	jn128(x0,x1,x2,x3)
    }
}
