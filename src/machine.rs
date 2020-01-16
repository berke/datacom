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

// pub struct &mut Machine(RefCell<Machine>);

// impl Copy for &mut Machine { }

// impl Clone for &mut Machine {
//     fn clone(&self)->&mut Machine {
// 	&mut Machine(self.0.clone())
//     }
// }

// // impl From<RefCell<Machine>> for &mut Machine {
// //     fn from(mac:RefCell<Machine>)->Self {
// // 	&mut Machine(mac)
// //     }
// // }

// impl &mut Machine {
//     pub fn new(mac:Machine)->Self {
// 	&mut Machine(RefCell::new(mac))
//     }

//     pub fn dump(self) {
// 	let mut mac = self.0.borrow_mut();
// 	mac.dump();
//     }
    
//     pub fn input(self,i:u16)->Index {
// 	let mut mac = self.0.borrow_mut();
// 	mac.get(&Gate::Input(i))
//     }

//     pub fn binop(self,op:Op,a:Index,b:Index)->Index {
// 	let mut mac = self.0.borrow_mut();
// 	mac.get(&Gate::Binop(op,a,b))
//     }

//     pub fn and(self,a:Index,b:Index)->Index {
// 	let mut mac = self.0.borrow_mut();
// 	mac.get(&Gate::Binop(Op::And,a,b))
//     }
//     pub fn or(self,a:Index,b:Index)->Index {
// 	let mut mac = self.0.borrow_mut();
// 	mac.get(&Gate::Binop(Op::Or,a,b))
//     }
//     pub fn not(self,a:Index)->Index {
// 	let mut mac = self.0.borrow_mut();
// 	mac.get(&Gate::Not(a))
//     }
//     // pub fn or(&mut self,a:Index,b:Index)->Index {
//     // 	self.get(&Gate::Binop(Op::Or,a,b))
//     // }
//     // pub fn xor(&mut self,a:Index,b:Index)->Index {
//     // 	self.get(&Gate::Binop(Op::Xor,a,b))
//     // }
//     // pub fn not(&mut self,a:Index)->Index {
//     // 	self.get(&Gate::Not(a))
//     // }
//     pub fn zero(&mut self)->Index {
// 	let mut mac = self.0.borrow_mut();
// 	mac.zero()
//     }
// }

#[derive(Clone)]
pub struct Machine {
    spec:RefCell<Vec<Gate>>,
    index:RefCell<BTreeMap<Gate,Index>>
}

impl Machine {
    pub fn dump(&self) {
	let spec = self.spec.borrow();
	for i in 0..spec.len() {
	    print!("x{} <- ",i);
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
	self.get(&Gate::Binop(op,a,b))
    }
    pub fn and(&self,a:Index,b:Index)->Index {
	self.get(&Gate::Binop(Op::And,a,b))
    }
    pub fn or(&self,a:Index,b:Index)->Index {
	self.get(&Gate::Binop(Op::Or,a,b))
    }
    pub fn xor(&self,a:Index,b:Index)->Index {
	self.get(&Gate::Binop(Op::Xor,a,b))
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
	    // W  = uvC + uVc + Uvc + UVC
	    //    = C(uv+UV) + c(uV + Uv)
	    // C' = uVC + UvC + UVc + UVC
	    //    = C(uV+Uv+UV) + cUV
	    let u = u[0];
	    let v = v[0];
	    let c = carry;

	    let and = |x,y| mac.and(x,y);
	    let or = |x,y| mac.or(x,y);
	    let not = |x| mac.not(x);

	    let w =
		or(,
		    and(c,or(,and(not(,u),not(,v)),and(u,v))),
		    and(not(,c),
			    or(,
				and(not(,u),v),
				and(u,not(,v)))));
	    let c2 =
		or(,
		    and(c,
			    or(,
				or(,
				    and(not(,u),v),
				    and(u,not(,v))),
				and(u,v))),
		    and(not(,c),
			    and(u,v)));
	    (Register(vec![w]),c2)
	} else {
	    let p = n / 2;
	    let q = n - p;
	    let u0 = self.slice(0,p);
	    let v0 = other.slice(0,p);
	    let mut u1 = self.slice(p,q);
	    let mut v1 = other.slice(p,q);
	    let (mut w0,c) = u0.add(mac,&v0,mac.zero());
	    let (mut w1,c2) = u1.add(mac,&v1,c);
	    w1.append(&mut w0);
	    (w1,c)
	};
	res
    }
}
