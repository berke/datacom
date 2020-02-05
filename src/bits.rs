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
	Bits{ n:32,b:vec![x as u64] }
    }
    pub fn new64(x:u64)->Self {
	Bits{ n:64,b:vec![x] }
    }
    // pub fn new128(x:u128)->Self {
    // 	Bits{ n:128,b:vec![(x >> 64) as u64,(x & ((1_u128<<64) - 1)) as u64] }
    // }

    pub fn len(&self)->usize {
	self.n
    }
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
    pub fn append_bits(&mut self,mut m:usize,mut x:u64) {
	let mut j;
	let mut k;
	let mut r; // Number of bits remaining in last word
	loop {
	    if m == 0 {
		// No more bits to append
		return;
	    }
	    j = self.n & 63;
	    k = self.n >> 6;
	    if j == 0 {
		// No bits remaining in the last word
		self.b.push(0);
		r = 64;
	    } else {
		r = 64 - j;
	    }
	    // Determine how many bits we can push
	    let s = r.min(m);
	    // Push the s least significant bits of x
	    // starting at position j
	    self.b[k] = self.b[k] | (flip64(x & ((1 << s) - 1)) >> (64 - s - j));
	    x >>= s;
	    m -= s;
	    self.n += s;
	}
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
	    write!(f,"{}",if self.get(self.n - 1 - j) { 1 } else { 0 })?
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

#[test]
fn test_bits() {
    let mut b = Bits::zero(0);
    let s = [3_usize,1,4,1,5,9,2,6,5,3,5,9];
    let mut bs = Vec::new();
    let mut x = true;
    for _ in 0..10 {
	for &si in s.iter() {
	    let bit = if x { !0 } else { 0 };
	    b.append_bits(si,bit);
	    for _ in 0..si {
		bs.push(x);
	    }
	    x = !x;
	}
    }
    let n = b.len();
    let ns = bs.len();
    if n != ns {
	panic!("Mismatch on length: n={} vs ns={}",n,ns);
    }
    for i in 0..n {
	if b.get(i) != bs[i] {
	    panic!("Mismatch on bit {}: {} vs {}",i,b.get(i),bs[i]);
	}
    }
    let mut xw = crate::xorwow::Xorwow::new(1);
    for _ in 0..100 {
	for k in 0..10000 {
	    let i = xw.next() as usize % n;
	    let bit = (xw.next() & 1) != 0;
	    b.set(i,bit);
	    bs[i] = bit;
	}
	for i in 0..n {
	    if b.get(i) != bs[i] {
		panic!("Mismatch on bit {}: {} vs {}",i,b.get(i),bs[i]);
	    }
	}
    }
}
