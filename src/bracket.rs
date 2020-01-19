use std::collections::BTreeMap;
use std::cell::{Cell,RefCell};
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

trait Morphism : Clone + Copy {
    type T : Clone + Copy;
    fn zero(self)->Self::T;
    fn one(self)->Self::T;
    fn add(self,a:Self::T,b:Self::T)->Self::T;
    fn mul(self,a:Self::T,b:Self::T)->Self::T;
}

impl Bracket {
    pub fn new()->Self {
	Bracket{
	    spec:RefCell::new(Vec::new()),
	    index:RefCell::new(BTreeMap::new()),
	    n_input:Cell::new(0)
	}
    }

    fn is_zero(&self,i:Index)->Ummo {
	match self.spec.borrow()[i as usize] {
	    Term::Atom(Atom::Zero) => Ummo::Yes,
	    Term::Atom(Atom::One) => Ummo::No,
	    _ => Ummo::Maybe
	}
    }

    fn is_one(&self,i:Index)->Ummo {
	match self.spec.borrow()[i as usize] {
	    Term::Atom(Atom::Zero) => Ummo::No,
	    Term::Atom(Atom::One) => Ummo::Yes,
	    _ => Ummo::Maybe
	}
    }

    fn eval_inner(&self,inputs:&Vec<bool>,busy:&mut Vec<bool>,done:&mut Vec<bool>,
		  value:&mut Vec<bool>,i:Index)->bool {
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
		Term::Atom(Atom::Zero) => false,
		Term::Atom(Atom::One) => true,
		Term::Atom(Atom::Var(j)) => inputs[j as usize],
		Term::Add(j1,j2) => {
		    let x1 = self.eval_inner(inputs,busy,done,value,j1);
		    let x2 = self.eval_inner(inputs,busy,done,value,j2);
		    x1 ^ x2
		},
		Term::Mul(j1,j2) => {
		    let x1 = self.eval_inner(inputs,busy,done,value,j1);
		    let x2 = self.eval_inner(inputs,busy,done,value,j2);
		    x1 & x2
		},
		Term::Term(j) => self.eval_inner(inputs,busy,done,value,j)
	    };
	done[i] = true;
	value[i] = x;
	busy[i] = false;
	x
    }

    fn eval_inner_morphism<M:Morphism>(&self,inputs:&Vec<M::T>,busy:&mut Vec<bool>,done:&mut Vec<bool>,
		  value:&mut Vec<M::T>,i:Index,phi:&M)->M::T {
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
		Term::Atom(Atom::Zero) => phi.zero(),
		Term::Atom(Atom::One) => phi.one(),
		Term::Atom(Atom::Var(j)) => inputs[j as usize],
		Term::Add(j1,j2) => {
		    let x1 = self.eval_inner_morphism(inputs,busy,done,value,j1,phi);
		    let x2 = self.eval_inner_morphism(inputs,busy,done,value,j2,phi);
		    phi.add(x1,x2)
		},
		Term::Mul(j1,j2) => {
		    let x1 = self.eval_inner_morphism(inputs,busy,done,value,j1,phi);
		    let x2 = self.eval_inner_morphism(inputs,busy,done,value,j2,phi);
		    phi.mul(x1,x2)
		},
		Term::Term(j) => self.eval_inner_morphism(inputs,busy,done,value,j,phi)
	    };
	done[i] = true;
	value[i] = x;
	busy[i] = false;
	x
    }

    fn eval_morphism<M:Morphism>(&self,constraints:&Vec<(Index,bool)>,phi:&M)->Vec<M::T> {
	let n = self.n_input.get() as usize;
	let mut inputs = Vec::new();
	let mut defined = Vec::new();
	let spec = self.spec.borrow();
	let m = spec.len();
	let zero = phi.zero();
	inputs.resize(n,zero);
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
			inputs[j] = if b { phi.one() } else { phi.zero() };
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
	value.resize(m,zero);
	for i in 0..m {
	    let _ = self.eval_inner_morphism(&inputs,&mut busy,&mut done,&mut value,i as Index,phi);
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
    fn dump(&self) {
	let spec = self.spec.borrow();
	for i in 0..spec.len() {
	    print!("t{} = ",i);
	    let v = &spec[i];
	    match v {
		Term::Atom(Atom::Zero) => println!("0"),
		Term::Atom(Atom::One) => println!("1"),
		Term::Atom(Atom::Var(i)) => println!("x{}",i),
		Term::Term(i) => println!("t{}",i),
		Term::Add(i,j) => println!("t{} + t{}",i,j),
		Term::Mul(i,j) => println!("t{}*t{}",i,j),
	    }
	}
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
