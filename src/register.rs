use crate::gate_soup::{Op,GateSoup,Index};
use crate::bits::Bits;

pub struct Register(Vec<Index>);

impl Register {
    pub fn dump<M:GateSoup>(_mac:&M,path:&str,
			    regs:Vec<(String,&Register)>)-> Result<(),std::io::Error> {
	use std::io::Write;
	let fd = std::fs::File::create(path)?;
	let mut fd = std::io::BufWriter::new(fd);

	for (name,Register(u)) in regs.iter() {
	    let n = u.len();
	    for i in 0..n {
		let j = u[i];
		writeln!(fd,"{} {} {}",name,n - 1 - i,j)?
		// match mac.as_input(u[i]) {
		//     None => panic!("Register bit {} of {} not an input",i,name),
		//     Some(j) => writeln!(fd,"{} {} {}",name,i,j)?
		// }
	    }
	}

	Ok(())
    }
	
    pub fn input<M:GateSoup>(mac:&M,n:Index)->Self {
	Register( (0..n).map(|_k| mac.new_input()).collect() )
    }

    pub fn constant<M:GateSoup>(mac:&M,bits:usize,x:u64)->Self {
	let zero = mac.zero();
	let one = mac.one();
	Register( (0..bits).map(|i| if (x >> (bits - 1 - i)) & 1 != 0 { one } else { zero }).collect() )
    }

    pub fn rotate_left(&self,s:usize)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| v[(k + s) % n]).collect())
    }

    pub fn shift_left(&self,s:usize,zero:Index)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| if k + s < n { v[k + s] } else { zero }).collect())
    }

    pub fn len(&self)->usize {
	self.0.len()
    }

    pub fn shift_right(&self,s:usize,zero:Index)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| if k >= s { v[k - s] } else { zero }).collect())
    }

    fn binop<M:GateSoup>(&self,mac:&M,op:Op,other:&Self)->Self {
	let Register(u) = &self;
	let Register(v) = &other;
	Register(u.iter().zip(v.iter()).map(|(ui,vi)| mac.binop(op,*ui,*vi)).collect())
    }

    pub fn not<M:GateSoup>(&self,mac:&M)->Self {
	let Register(u) = &self;
	Register(u.iter().map(|ui| mac.not(*ui)).collect())
    }

    pub fn bit(&self,i:usize)->Index {
	let Register(u) = &self;
	let n = u.len();
	u[n - 1 - i]
    }

    pub fn set_bit(&mut self,i:usize,bit:Index) {
	let n = self.0.len();
	self.0[n - 1 - i] = bit;
    }

    pub fn scale<M:GateSoup>(&self,mac:&M,bit:Index)->Self {
	let Register(u) = &self;
	Register(u.iter().map(|&ui| mac.and(bit,ui)).collect())
    }

    pub fn and<M:GateSoup>(&self,mac:&M,other:&Self)->Self {
	self.binop(mac,Op::And,other)
    }

    pub fn or<M:GateSoup>(&self,mac:&M,other:&Self)->Self {
	self.binop(mac,Op::Or,other)
    }

    pub fn xor<M:GateSoup>(&self,mac:&M,other:&Self)->Self {
	self.binop(mac,Op::Xor,other)
    }

    pub fn slice(&self,j0:usize,n:usize)->Self {
	let Register(u) = &self;
	Register(Vec::from(&u[j0..j0+n]))
    }

    pub fn clone(&self)->Self {
	Register(self.0.clone())
    }

    pub fn join(self:&Self,other:&Self)->Self {
	let Register(ref u) = self;
	let Register(ref v) = other;
	let mut u2 = u.clone();
	u2.append(&mut v.clone());
	Register(u2)
    }

    pub fn append(self:&mut Self,other:&mut Self) {
	let Register(ref mut u) = self;
	let Register(ref mut v) = other;
	u.append(v);
    }

    pub fn constraints(&self,x:u64)->Vec<(Index,bool)> {
	let n = self.0.len();
	self.0.iter().enumerate().map(|(i,&u)| (u,(x >> (n - 1 - i)) & 1 != 0)).collect()
    }

    pub fn constraints_from_bits(&self,x:&Bits)->Vec<(Index,bool)> {
	let n = self.0.len();
	self.0.iter().enumerate().map(|(i,&u)| (u,x.get(i))).collect()
    }

    pub fn value(&self,values:&Vec<bool>)->u64 {
	let mut q = 0;
	let n = self.0.len();
	for i in 0..n {
	    q <<= 1;
	    if values[self.0[i] as usize] {
		q |= 1;
	    }
	}
	q
    }

    pub fn value_as_bits(&self,values:&Vec<bool>)->Bits {
	let n = self.0.len();
	let mut b = Bits::zero(n);
	for i in 0..n {
	    if values[self.0[i] as usize] {
		b.set(i,true);
	    }
	}
	b
    }

    pub fn decoder<M:GateSoup>(&self,mac:&mut M)->Self {
	let Register(u) = &self;
	match u.len() {
	    0 => Register(vec![]),
	    1 => Register(vec![u[0],mac.not(u[0])]),
	    n => {
		let d = self.slice(1,n-1).decoder(mac);
		let mut d0 = d.scale(mac,mac.not(u[0]));
		let mut d1 = d.scale(mac,u[0]);
		d1.append(&mut d0);
		d1
	    }
	}
    }

    pub fn add<M:GateSoup>(&self,mac:&mut M,other:&Self,carry:Index)->(Self,Index) {
	let Register(u) = &self;
	let Register(v) = &other;
	let n = u.len();
	if v.len() != n {
	    panic!("Mismatched register sizes for add, {} vs {}",n,v.len());
	}
	let res =
	if n == 1 {
	    // U V C | W C'
	    // ============
	    // 0 0 0 | 0 0
	    // 0 0 1 | 1 0
	    // 0 1 0 | 1 0
	    // 0 1 1 | 0 1
	    // 1 0 0 | 1 0
	    // 1 0 1 | 0 1
	    // 1 1 0 | 0 1
	    // 1 1 1 | 1 1
	    //
	    // W  = uvC + uVc + Uvc + UVC       -- ok
	    //    = C(uv+UV) + c(uV + Uv)       -- ok
	    // C' = uVC + UvC + UVc + UVC       -- ok
	    //    = C(uV+Uv+UV) + cUV           -- ok
	    //    = C(!uv) + cUV                -- ok
	    let u = u[0];
	    let v = v[0];
	    let c = carry;

	    let and = |x,y| mac.and(x,y);
	    let or = |x,y| mac.or(x,y);
	    let not = |x| mac.not(x);

	    let _c = not(c);
	    let uv = and(u,v);
	    let u_v = and(u,not(v));
	    let _uv = and(not(u),v);
	    let _u_v = and(not(u),not(v));

	    let w =
		or(and(c,or(_u_v,uv)),
		   and(_c,or(_uv,u_v)));
	    let c2 =
		or(and(c,not(_u_v)),
		   and(_c,uv));
	    (Register(vec![w]),c2)
	} else {
	    let p = n / 2;
	    let q = n - p;
	    // self  ->   [u0 u1]
	    // other ->   [v0 v1]
	    //       -> c [w0 w1] carry
	    let u0 = self.slice(0,p);
	    let v0 = other.slice(0,p);
	    let u1 = self.slice(p,q);
	    let v1 = other.slice(p,q);
	    let (mut w1,c1) = u1.add(mac,&v1,carry);
	    let (mut w0,c0) = u0.add(mac,&v0,c1);
	    w0.append(&mut w1);
	    (w0,c0)
	};
	res
    }

    pub fn all_ones<M:GateSoup>(&self,mac:&mut M)->Index {
	let Register(u) = &self;
	let n = u.len();
	match n {
	    1 => u[0],
	    2 => mac.and(u[0],u[1]),
	    _ => {
		let p = n / 2;
		let q = n - p;
		// self  ->   [u0 u1]
		// other ->   [v0 v1]
		let u0 = self.slice(0,p);
		let u1 = self.slice(p,q);
		let x0 = u0.all_ones(mac);
		let x1 = u1.all_ones(mac);
		mac.and(x0,x1)
	    }
	}
    }
}
