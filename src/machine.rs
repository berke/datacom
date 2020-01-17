use std::collections::BTreeMap;
use std::cell::RefCell;
use std::borrow::BorrowMut;

pub type Index = u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gate {
    Zero,
    Input(u16),
    Not(Index),
    Binop(Op,Index,Index)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Op {
    And = 0,
    Or = 1,
    Xor = 2
}

#[derive(Clone)]
pub struct Machine {
    spec:RefCell<Vec<Gate>>,
    index:RefCell<BTreeMap<Gate,Index>>
}

impl Machine {
    pub fn dump(&self) {
	let spec = self.spec.borrow();
	for i in 0..spec.len() {
	    print!("x{} <- ",i+1);
	    let v = &spec[i];
	    match v {
		Gate::Zero => println!("0"),
		Gate::Input(i) => println!("INPUT({})",i),
		Gate::Not(i) => println!("!x{}",i),
		Gate::Binop(Op::And,i,j) => println!("x{} & x{}",i,j),
		Gate::Binop(Op::Or,i,j) => println!("x{} | x{}",i,j),
		Gate::Binop(Op::Xor,i,j) => println!("x{} ^ x{}",i,j)
	    }
	}
    }

    pub fn num_clauses(&self,constraints:&Vec<(Index,bool)>)->usize {
	let mut cnt = 0;
	let sp  = self.spec.borrow();
	for i0 in 0..sp.len() {
	    let i = i0 as u16;
	    cnt +=
		match sp[i0] {
		    Gate::Zero => 1,
		    Gate::Input(_) => 0,
		    Gate::Not(j) => 2,
		    Gate::Binop(Op::And,j1,j2) => 4,
		    Gate::Binop(Op::Or,j1,j2) => 4,
		    Gate::Binop(Op::Xor,j1,j2) => 4
		}
	}
	cnt += constraints.len();
	cnt
    }

    pub fn save_cnf(&self,path:&str,constraints:&Vec<(Index,bool)>)->Result<(),std::io::Error> {
	use std::io::Write;
	let fd = std::fs::File::create(path)?;
	let mut fd = std::io::BufWriter::new(fd);
	let sp  = self.spec.borrow();
	let m = self.num_clauses(constraints);
	let n = sp.len();
	write!(fd,"p cnf {} {}\n",m,n);
	let pos = |i| (i + 1) as i32;
	let neg = |i| -((i + 1) as i32);
	for i0 in 0..sp.len() {
	    let z = i0 as u16;
	    match sp[i0] {
		Gate::Zero => write!(fd,"{} 0\n",neg(z))?,
		Gate::Input(_) => (),
		// y = !x
		// x y w o
		// -------   
		// 0 0 1 0
		// 0 1 1 1
		// 1 0 0 1
		// 1 1 0 0
		// (-x-y)(x+y)
		Gate::Not(x) => {
		    write!(fd,"{} {} 0\n",pos(x),pos(z))?;
		    write!(fd,"{} {} 0\n",neg(x),neg(z))?;
		},
		// z = x & y
		// x y z w o
		// ---------
		// 0 0 0 0 1
		// 0 0 1 0 0
		// 0 1 0 0 1
		// 0 1 1 0 0
		// 1 0 0 0 1
		// 1 0 1 0 0
		// 1 1 0 1 0
		// 1 1 1 1 1
		Gate::Binop(Op::And,x,y) => {
		    write!(fd,"{} {} {} 0\n",pos(x),pos(y),neg(z))?;
		    write!(fd,"{} {} {} 0\n",pos(x),neg(y),neg(z))?;
		    write!(fd,"{} {} {} 0\n",neg(x),pos(y),neg(z))?;
		    write!(fd,"{} {} {} 0\n",neg(x),neg(y),pos(z))?;
		},
		// z = x | y
		// x y z w o
		// ---------
		// 0 0 0 0 1
		// 0 0 1 0 0
		// 0 1 0 1 0
		// 0 1 1 1 1
		// 1 0 0 1 0
		// 1 0 1 1 1
		// 1 1 0 1 0
		// 1 1 1 1 1
		Gate::Binop(Op::Or,x,y) => {
		    write!(fd,"{} {} {} 0\n",pos(x),pos(y),neg(z))?;
		    write!(fd,"{} {} {} 0\n",pos(x),neg(y),pos(z))?;
		    write!(fd,"{} {} {} 0\n",neg(x),pos(y),pos(z))?;
		    write!(fd,"{} {} {} 0\n",neg(x),neg(y),pos(z))?;
		},
		
		// ENCODE z = x ^ y
		// w := x^z
		// o := (w = z)
		// x y w z o
		// ---------
		// 0 0 0 0 1
		// 0 0 0 1 0 (1) -x -y +z
		// 0 1 1 0 0 (2) -x +y -z
		// 0 1 1 1 1
		// 1 0 1 0 0 (3) +x -y -z
		// 1 0 1 1 1
		// 1 1 0 0 1
		// 1 1 0 1 0 (4) +x +y +z
		//
		// (1)     (2)     (3)    (4)
		// (-x-y+z)(-x+y-z)(x-y-z)(x+y+z)
		Gate::Binop(Op::Xor,x,y) => {
		    write!(fd,"{} {} {} 0\n",pos(x),pos(y),neg(z))?;
		    write!(fd,"{} {} {} 0\n",pos(x),neg(y),pos(z))?;
		    write!(fd,"{} {} {} 0\n",neg(x),pos(y),pos(z))?;
		    write!(fd,"{} {} {} 0\n",neg(x),neg(y),neg(z))?;
		}
	    }
	}
	for &(i,b) in constraints.iter() {
	    write!(fd,"{} 0\n",if b { pos(i) } else { neg(i) })?;
	}
	Ok(())
    }
    pub fn new()->Self {
	Machine{
	    spec:RefCell::new(Vec::new()),
	    index:RefCell::new(BTreeMap::new())
	}
    }
    pub fn find(&self,b:&Gate)->Option<Index> {
	self.index.borrow().get(b).map(|x| *x)
    }
    // commutation - canonicalization

    pub fn get(&self,b:&Gate)->Index {
	match self.find(b) {
	    Some(i) => i,
	    None => {
		let mut spec = self.spec.borrow_mut();
		let i = spec.len() as u16;
		spec.push(*b);
		self.index.borrow_mut().insert(*b,i);
		i
	    }
	}
    }
    pub fn input(&self,i:u16)->Index {
	self.get(&Gate::Input(i))
    }
    pub fn binop(&self,op:Op,a:Index,b:Index)->Index {
	let (a,b) = (a.min(b),a.max(b));
	self.get(&Gate::Binop(op,a,b))
    }
    pub fn and(&self,a:Index,b:Index)->Index {
	if a == b {
	    a
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    self.get(&Gate::Binop(Op::And,a,b))
	}
    }
    pub fn or(&self,a:Index,b:Index)->Index {
	if a == b {
	    a
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    self.get(&Gate::Binop(Op::Or,a,b))
	}
    }
    pub fn xor(&self,a:Index,b:Index)->Index {
	if a == b {
	    self.zero()
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    self.get(&Gate::Binop(Op::Xor,a,b))
	}
    }
    pub fn not(&self,a:Index)->Index {
	self.get(&Gate::Not(a))
    }
    pub fn zero(&self)->Index {
	self.get(&Gate::Zero)
    }
}

pub struct Register(Vec<Index>);

impl Register {
    pub fn input(mac:&mut Machine,k0:u16,n:u16)->Self {
	Register( (k0..k0+n).map(|k| mac.input(k as u16)).collect() )
    }
    pub fn rotate_left(self:&Register,s:usize)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| v[(k + s) % n]).collect())
    }

    fn binop(self:&Register,mac:&mut Machine,op:Op,other:&Register)->Register {
	let Register(u) = &self;
	let Register(v) = &other;
	let n = u.len();
	Register(u.iter().zip(v.iter()).map(|(ui,vi)| mac.binop(op,*ui,*vi)).collect())
    }

    pub fn and(self:&Register,mac:&mut Machine,other:&Register)->Register {
	self.binop(mac,Op::And,other)
    }

    pub fn or(self:&Register,mac:&mut Machine,other:&Register)->Register {
	self.binop(mac,Op::Or,other)
    }

    pub fn xor(self:&Register,mac:&mut Machine,other:&Register)->Register {
	self.binop(mac,Op::Xor,other)
    }

    pub fn slice(self:&Register,j0:usize,n:usize)->Register {
	let Register(u) = &self;
	Register(Vec::from(&u[j0..j0+n]))
    }

    pub fn append(self:&mut Register,other:&mut Register) {
	let Register(ref mut u) = self;
	let Register(ref mut v) = other;
	u.append(v);
    }

    pub fn constraints(self:&Register,x:u64)->Vec<(Index,bool)> {
	let n = self.0.len();
	self.0.iter().enumerate().map(|(i,&u)| (u,(x >> (n - 1 - i)) & 1 != 0)).collect()
    }

    pub fn add(self:&Register,mac:&mut Machine,other:&Register,carry:Index)->(Register,Index) {
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
	    let u0 = self.slice(0,p);
	    let v0 = other.slice(0,p);
	    let mut u1 = self.slice(p,q);
	    let mut v1 = other.slice(p,q);
	    let (mut w1,c) = u1.add(mac,&v1,carry);
	    let (mut w0,c0) = u0.add(mac,&v0,c);
	    w0.append(&mut w1);
	    (w0,c)
	};
	res
    }
}
