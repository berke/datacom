use std::fmt;

#[derive(Clone)]
pub struct Bits {
    n:usize,
    b:Vec<u64>
}

pub fn flip64(x:u64)->u64 {
    let mut y = x;
    y = ((y >> 32) & 0x00000000ffffffff) | ((y & 0x00000000ffffffff) << 32);
    y = ((y >> 16) & 0x0000ffff0000ffff) | ((y & 0x0000ffff0000ffff) << 16);
    y = ((y >>  8) & 0x00ff00ff00ff00ff) | ((y & 0x00ff00ff00ff00ff) <<  8);
    y = ((y >>  4) & 0x0f0f0f0f0f0f0f0f) | ((y & 0x0f0f0f0f0f0f0f0f) <<  4);
    y = ((y >>  2) & 0x3333333333333333) | ((y & 0x3333333333333333) <<  2);
    y = ((y >>  1) & 0x5555555555555555) | ((y & 0x5555555555555555) <<  1);
    y
}

pub fn flip32(x:u32)->u32 {
    let mut y = x;
    y = ((y >> 16) & 0x0000ffff) | ((y & 0x0000ffff) << 16);
    y = ((y >>  8) & 0x00ff00ff) | ((y & 0x00ff00ff) <<  8);
    y = ((y >>  4) & 0x0f0f0f0f) | ((y & 0x0f0f0f0f) <<  4);
    y = ((y >>  2) & 0x33333333) | ((y & 0x33333333) <<  2);
    y = ((y >>  1) & 0x55555555) | ((y & 0x55555555) <<  1);
    y
}

impl Bits {
    pub fn new32(x:u32)->Self {
	Bits{ n:32,b:vec![flip32(x) as u64] }
    }
    pub fn new64(x:u64)->Self {
	Bits{ n:64,b:vec![flip64(x)] }
    }
    // pub fn new128(x:u128)->Self {
    // 	Bits{ n:128,b:vec![(x >> 64) as u64,(x & ((1_u128<<64) - 1)) as u64] }
    // }
    pub fn zero(n:usize)->Self {
	let mut b = Vec::new();
	b.resize((n+63)>>6,0);
	Bits{ n,b }
    }
    pub fn append_bit(&mut self,b:bool) {
	let j = self.n & 63;
	if j != 0 {
	    if b {
		self.b[self.n >> 6] |= 1_u64 << j;
	    }
	} else {
	    self.b.push(if b { 1 } else { 0 })
	}
	self.n += 1;
    }
    pub fn get(&self,i:usize)->bool {
	if i >= self.n {
	    panic!("Bit {} out-of-range for a bit vector of size {}",i,self.n);
	}
	(self.b[i >> 6] >> (i & 63)) & 1 != 0
    }
    pub fn set(&mut self,i:usize,b:bool) {
	if i >= self.n {
	    panic!("Bit {} out-of-range for a bit vector of size {}",i,self.n);
	}
	let m = 1 << (i & 63);
	if b {
	    self.b[i >> 6] |= m;
	} else {
	    self.b[i >> 6] &= !m;
	}
    }
    pub fn append(&mut self,other:&Self) {
	for i in 0..other.n {
	    self.append_bit(other.get(i));
	}
    }
    pub fn concat(v:&Vec<Self>)->Self {
	let mut b = Self::zero(0);
	for c in v.iter() {
	    b.append(c);
	}
	b
    }
}

impl fmt::Debug for Bits {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
	for j in 0..self.n {
	    write!(f,"{}",if self.get(j) { 1 } else { 0 })?
	}
	Ok(())
    }
}

impl PartialEq for Bits {
    fn eq(&self,other:&Self)->bool {
	if self.n == other.n {
	    self.b.iter().zip(other.b.iter()).all(|(x,y)| x == y)
	} else {
	    false
	}
    }
}
