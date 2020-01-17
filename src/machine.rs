use std::collections::BTreeMap;
use std::cell::{Cell,RefCell};

type Index = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gate {
    Zero,
    Input(Index),
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
    index:RefCell<BTreeMap<Gate,Index>>,
    n_input:Cell<Index>

}

impl Machine {
    pub fn eval(&self,constraints:&Vec<(Index,bool)>)->Vec<bool> {
	let n = self.n_input.get() as usize;
	let mut inputs = Vec::new();
	let mut defined = Vec::new();
	let spec = self.spec.borrow();
	let m = spec.len();
	inputs.resize(n,false);
	defined.resize(n,false);
	let mut n_ign_constr = 0;
	for &(i,b) in constraints.iter() {
	    let i = i as usize;
	    match spec[i] {
		Gate::Input(j) => {
		    let j = j as usize;
		    if defined[j] {
			println!("Multiply defined input {}",i)
		    } else {
			defined[j] = true;
			inputs[j] = b;
		    }
		},
		_ => n_ign_constr += 1
	    }
	}
	if n_ign_constr > 0 {
	    println!("Warning: Number of ignored constraints on non-input gates: {}",n_ign_constr);
	}
	for i in 0..n {
	    if !defined[i] {
		panic!("Input {} not defined",i);
	    }
	}
	let mut busy = Vec::new();
	let mut done = Vec::new();
	let mut value = Vec::new();
	busy.resize(m,false);
	done.resize(m,false);
	value.resize(m,false);
	for i in 0..m {
	    let _ = self.eval_inner(&inputs,&mut busy,&mut done,&mut value,i as Index);
	};
	value
    }
    fn eval_inner(&self,inputs:&Vec<bool>,busy:&mut Vec<bool>,done:&mut Vec<bool>,value:&mut Vec<bool>,i:Index)->bool {
	let i = i as usize;
	if busy[i] {
	    panic!("Circular dependency involving gate {}",i);
	}
	if done[i] {
	    return value[i];
	}
	busy[i] = true;
	let x =
	    match self.spec.borrow()[i] {
		Gate::Input(j) => inputs[j as usize],
		Gate::Not(j) => !self.eval_inner(inputs,busy,done,value,j),
		Gate::Binop(op,j1,j2) => {
		    let x1 = self.eval_inner(inputs,busy,done,value,j1);
		    let x2 = self.eval_inner(inputs,busy,done,value,j2);
		    match op {
			Op::And => x1 & x2,
			Op::Or => x1 | x2,
			Op::Xor => x1 ^ x2
		    }
		},
		Gate::Zero => false
	    };
	done[i] = true;
	value[i] = x;
	busy[i] = false;
	x
    }
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
	    cnt +=
		match sp[i0] {
		    Gate::Zero => 1,
		    Gate::Input(_) => 0,
		    Gate::Not(_) => 2,
		    Gate::Binop(Op::And,_,_) => 4,
		    Gate::Binop(Op::Or,_,_) => 4,
		    Gate::Binop(Op::Xor,_,_) => 4
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
	write!(fd,"p cnf {} {}\n",m,n)?;
	let pos = |i| (i + 1) as i32;
	let neg = |i| -((i + 1) as i32);
	for i0 in 0..sp.len() {
	    let z = i0 as Index;
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
	    index:RefCell::new(BTreeMap::new()),
	    n_input:Cell::new(0)
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
		let i = spec.len() as Index;
		spec.push(*b);
		self.index.borrow_mut().insert(*b,i);
		i
	    }
	}
    }
    pub fn input(&self,i:Index)->Index {
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
    pub fn input(mac:&mut Machine,n:Index)->Self {
	let k0 = mac.n_input.get();
	mac.n_input.set(k0 + n);
	Register( (k0..k0+n).map(|k| mac.input(k as Index)).collect() )
    }

    pub fn rotate_left(self:&Register,s:usize)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| v[(k + s) % n]).collect())
    }

    pub fn shift_left(self:&Register,s:usize,zero:Index)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| if k + s < n { v[k + s] } else { zero }).collect())
    }

    pub fn shift_right(self:&Register,s:usize,zero:Index)->Self {
	let Register(v) = &self;
	let n = v.len();
	Register( (0..n).map(|k| if k >= s { v[k - s] } else { zero }).collect())
    }

    fn binop(self:&Register,mac:&mut Machine,op:Op,other:&Register)->Register {
	let Register(u) = &self;
	let Register(v) = &other;
	Register(u.iter().zip(v.iter()).map(|(ui,vi)| mac.binop(op,*ui,*vi)).collect())
    }

    pub fn bit(self:&Register,i:usize)->Index {
	let Register(u) = &self;
	u[i]
    }

    pub fn and_bit(self:&Register,mac:&mut Machine,bit:Index)->Register {
	let Register(u) = &self;
	Register(u.iter().map(|&ui| mac.and(bit,ui)).collect())
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

    pub fn clone(self:&Register)->Register {
	Register(self.0.clone())
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

    pub fn value(self:&Register,values:&Vec<bool>)->u64 {
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
}
