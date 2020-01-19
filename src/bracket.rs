use std::collections::{BTreeMap,BTreeSet};
use std::cell::{Cell,RefCell};
use std::rc::Rc;
use crate::gate_soup::{Index,Op,GateSoup};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    Atom(Atom),
    Term(Index),
    Add(Index,Index),
    Mul(Index,Index)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Atom {
    Zero,
    One,
    Var(Index)
}

#[derive(Clone)]
pub struct Bracket {
    spec:RefCell<Vec<Term>>,
    index:RefCell<BTreeMap<Term,Index>>,
    n_input:Cell<Index>
}

enum Ummo {
    Yes,
    No,
    Maybe,
    Bullshit
}

pub trait Morphism {
    type T : Clone;
    fn zero(&self)->Self::T;
    fn one(&self)->Self::T;
    fn input(&self,i:Index)->Self::T;
    fn add(&self,a:Self::T,b:Self::T)->Self::T;
    fn mul(&self,a:Self::T,b:Self::T)->Self::T;
}

pub struct StandardMorphism { }

impl Morphism for StandardMorphism {
    type T = bool;
    fn zero(&self)->Self::T { false }
    fn one(&self)->Self::T { true }
    fn input(&self,i:Index)->Self::T { panic!("Undefined input") }
    fn add(&self,a:Self::T,b:Self::T)->Self::T { a ^ b }
    fn mul(&self,a:Self::T,b:Self::T)->Self::T { a & b }
}

impl StandardMorphism {
    pub fn new()->Self {
	StandardMorphism{ }
    }
}

pub struct SizeMorphism { }

impl Morphism for SizeMorphism {
    type T = f64;
    fn zero(&self)->Self::T { 1.0 }
    fn one(&self)->Self::T { 1.0 }
    fn input(&self,i:Index)->Self::T { 1.0 }
    fn add(&self,a:Self::T,b:Self::T)->Self::T { a + b }
    fn mul(&self,a:Self::T,b:Self::T)->Self::T { a + b }
}

impl SizeMorphism {
    pub fn new()->Self {
	SizeMorphism{ }
    }
}

pub struct InputSetMorphism { }

impl Morphism for InputSetMorphism {
    type T = u128;
    fn zero(&self)->Self::T { 0 }
    fn one(&self)->Self::T { 0 }
    fn input(&self,i:Index)->Self::T { println!("I{}",i); 1 << i }
    fn add(&self,a:Self::T,b:Self::T)->Self::T { a | b }
    fn mul(&self,a:Self::T,b:Self::T)->Self::T { a | b }
}

impl InputSetMorphism {
    pub fn new()->Self {
	InputSetMorphism{ }
    }
}

pub struct TrimmedMorphism { mw:u32 }

impl Morphism for TrimmedMorphism {
    type T = (bool,Rc<BTreeSet<u128>>);
    fn zero(&self)->Self::T { (false,Rc::new(BTreeSet::new())) }
    fn one(&self)->Self::T { (true,Rc::new(BTreeSet::new())) }
    fn input(&self,i:Index)->Self::T {
	let mut bt = BTreeSet::new();
	bt.insert(1 << i);
	(false,Rc::new(bt))
    }
    fn add(&self,a:Self::T,b:Self::T)->Self::T {
	let c = a.1.symmetric_difference(&b.1).cloned().collect();
	(a.0^b.0,Rc::new(c))
    }
    fn mul(&self,a:Self::T,b:Self::T)->Self::T {
	println!("{}",a.1.len());
	let mut c = (*(a.1)).clone();
	for &ai in a.1.iter() {
	    for &bi in b.1.iter() {
		let ci = ai | bi;
		if ci.count_ones() <= self.mw {
		    if c.contains(&ci) {
			c.remove(&ci);
		    } else {
			c.insert(ci);
		    }
		}
	    }
	}
	if a.0 {
	    c.append(&mut (*(b.1)).clone());
	}
	if b.0 {
	    c.append(&mut (*(a.1)).clone());
	}
	let mut d = BTreeSet::new();
	for &ci in c.iter() {
	    if ci.count_ones() <= self.mw {
		d.insert(ci);
	    }
	}
	(a.0 & b.0,Rc::new(d))
	// (1 + a)(1 + b) = 1 + b + a + ab
	// a(1 + b) = a + ab
	// (1 + a)b = b + ab
	// ab = ab
    }
}

impl TrimmedMorphism {
    pub fn new(mw:u32)->Self {
	TrimmedMorphism{ mw }
    }
    pub fn dump(&self,a:&<TrimmedMorphism as Morphism>::T) {
	let mut first = true;
	if a.0 {
	    print!("1");
	    first = false;
	}
	for ai in a.1.iter() {
	    if !first {
		print!(" + ");
	    } else {
		first = false;
	    }
	    let mut x = *ai;
	    loop {
		let n = x.trailing_zeros();
		if n < 128 {
		    print!("x{}",n);
		    x &= !(1 << n);
		} else {
		    break;
		}
	    }
	}
	println!("");
    }
}

impl Bracket {
    pub fn new()->Self {
	Bracket{
	    spec:RefCell::new(Vec::new()),
	    index:RefCell::new(BTreeMap::new()),
	    n_input:Cell::new(0)
	}
    }

    pub fn is_zero(&self,i:Index)->Ummo {
	match self.spec.borrow()[i as usize] {
	    Term::Atom(Atom::Zero) => Ummo::Yes,
	    Term::Atom(Atom::One) => Ummo::No,
	    _ => Ummo::Maybe
	}
    }

    pub fn is_one(&self,i:Index)->Ummo {
	match self.spec.borrow()[i as usize] {
	    Term::Atom(Atom::Zero) => Ummo::No,
	    Term::Atom(Atom::One) => Ummo::Yes,
	    _ => Ummo::Maybe
	}
    }

    pub fn eval_inner_morphism<M:Morphism>(&self,
					   defined:&Vec<bool>,
					   inputs:&Vec<bool>,
					   busy:&mut Vec<bool>,done:&mut Vec<bool>,
		  value:&mut Vec<M::T>,i:Index,phi:&M)->M::T {
	let i = i as usize;
	if busy[i] {
	    panic!("Circular dependency involving gate {}",i);
	}
	if done[i] {
	    return value[i].clone();
	}
	busy[i] = true;
	let x =
	    match self.spec.borrow()[i] {
		Term::Atom(Atom::Zero) => phi.zero(),
		Term::Atom(Atom::One) => phi.one(),
		Term::Atom(Atom::Var(j)) =>
		    if defined[j as usize] {
			if inputs[j as usize] {
			    phi.one()
			} else {
			    phi.zero()
			}
		    } else {
			phi.input(j)
		    }
		Term::Add(j1,j2) => {
		    let x1 = self.eval_inner_morphism(defined,inputs,busy,done,value,j1,phi);
		    let x2 = self.eval_inner_morphism(defined,inputs,busy,done,value,j2,phi);
		    phi.add(x1,x2)
		},
		Term::Mul(j1,j2) => {
		    let x1 = self.eval_inner_morphism(defined,inputs,busy,done,value,j1,phi);
		    let x2 = self.eval_inner_morphism(defined,inputs,busy,done,value,j2,phi);
		    phi.mul(x1,x2)
		},
		Term::Term(j) => self.eval_inner_morphism(defined,inputs,busy,done,value,j,phi)
	    };
	done[i] = true;
	value[i] = x.clone();
	busy[i] = false;
	x
    }

    pub fn eval_morphism<M:Morphism>(&self,constraints:&Vec<(Index,bool)>,phi:&M)->Vec<M::T> {
	let n = self.num_inputs();
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
		Term::Atom(Atom::Var(j)) => {
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
	let mut busy = Vec::new();
	let mut done = Vec::new();
	let mut value = Vec::new();
	busy.resize(m,false);
	done.resize(m,false);
	let zero = phi.zero();
	value.resize(m,zero);
	for i in 0..m {
	    let _ = self.eval_inner_morphism(&defined,&inputs,&mut busy,&mut done,&mut value,i as Index,phi);
	};
	value
    }

    fn find(&self,b:&Term)->Option<Index> {
	self.index.borrow().get(b).map(|x| *x)
    }
    // commutation - canonicalization

    fn get(&self,b:&Term)->Index {
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
}

impl GateSoup for Bracket {
    fn eval(&self,constraints:&Vec<(Index,bool)>)->Vec<bool> {
	let phi = StandardMorphism::new();
	self.eval_morphism(constraints,&phi)
    }
    fn dump(&self,path:&str)->Result<(),std::io::Error> {
	use std::io::Write;
	let fd = std::fs::File::create(path)?;
	let mut fd = std::io::BufWriter::new(fd);
	let spec = self.spec.borrow();
	for i in 0..spec.len() {
	    write!(fd,"t{} = ",i)?;
	    let v = &spec[i];
	    match v {
		Term::Atom(Atom::Zero) => writeln!(fd,"0")?,
		Term::Atom(Atom::One) => writeln!(fd,"1")?,
		Term::Atom(Atom::Var(i)) => writeln!(fd,"x{}",i)?,
		Term::Term(i) => writeln!(fd,"t{}",i)?,
		Term::Add(i,j) => writeln!(fd,"t{} + t{}",i,j)?,
		Term::Mul(i,j) => writeln!(fd,"t{}*t{}",i,j)?,
	    }
	}
	Ok(())
    }

    fn num_inputs(&self)->usize {
	self.n_input.get() as usize
    }

    fn new_input(&mut self)->Index {
	let i = self.n_input.get();
	self.n_input.set(i + 1);
	self.get(&Term::Atom(Atom::Var(i)))
    }
    fn input(&self,i:Index)->Index {
	self.get(&Term::Atom(Atom::Var(i)))
    }
    fn binop(&self,op:Op,a:Index,b:Index)->Index {
	let (a,b) = (a.min(b),a.max(b));
	match op {
	    Op::Xor => self.xor(a,b),
	    Op::And => self.and(a,b),
	    Op::Or => self.or(a,b)
	}
    }
    fn and(&self,a:Index,b:Index)->Index {
	if a == b {
	    a
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    match (self.is_one(a),self.is_one(b)) {
		(Ummo::Yes,_) => b,
		(_,Ummo::Yes) => a,
		(Ummo::Yes,Ummo::Yes) => a,
		(Ummo::No,_)|(_,Ummo::No) => self.zero(),
		(_,_) => self.get(&Term::Mul(a,b))
	    }
	}
    }
    fn or(&self,a:Index,b:Index)->Index {
	// a | b = a+b+ab
	if a == b {
	    a
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    match (self.is_one(a),self.is_one(b)) {
		(Ummo::Yes,_)|(_,Ummo::Yes) => self.one(),
		(_,Ummo::No) => a,
		(Ummo::No,_) => b,
		(_,_) => {
		    let c = self.xor(a,b);
		    let d = self.and(a,b);
		    self.xor(c,d)
		}
	    }
	}
    }
    fn xor(&self,a:Index,b:Index)->Index {
	if a == b {
	    self.zero()
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    match (self.is_one(a),self.is_one(b)) {
		(_,Ummo::No) => a,
		(Ummo::No,_) => b,
		(_,_) => self.get(&Term::Add(a,b))
	    }
	}
    }
    fn not(&self,a:Index)->Index {
	let one = self.one();
	self.get(&Term::Add(a,one))
    }
    fn zero(&self)->Index {
	self.get(&Term::Atom(Atom::Zero))
    }
    fn one(&self)->Index {
	self.get(&Term::Atom(Atom::One))
    }
}
