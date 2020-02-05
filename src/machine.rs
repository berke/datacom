use std::collections::BTreeMap;
use std::cell::{Cell,RefCell};
use crate::gate_soup::{InputIndex,Index,Op,GateSoup};
use cryptominisat::{Solver,Lit};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gate {
    Zero,
    Input(InputIndex),
    Not(Index),
    Binop(Op,Index,Index)
}

#[derive(Clone)]
pub struct Machine {
    spec:RefCell<Vec<Gate>>,
    index:RefCell<BTreeMap<Gate,Index>>,
    n_input:Cell<Index>

}

impl Machine {
    pub fn new()->Self {
	Machine{
	    spec:RefCell::new(Vec::new()),
	    index:RefCell::new(BTreeMap::new()),
	    n_input:Cell::new(0)
	}
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

    fn num_clauses(&self,constraints:&Vec<(Index,bool)>)->usize {
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

    pub fn solver(&self,constraints:&Vec<(Index,bool)>)->Solver {
	let mut solver = Solver::new();
	let sp  = self.spec.borrow();
	let n = sp.len();
	solver.new_vars(n);

	let pos = |i| Lit::new(i,false).unwrap();
	let neg = |i| Lit::new(i,true).unwrap();
	let use_xor = true;
	for i0 in 0..sp.len() {
	    let z = i0 as Index;
	    match sp[i0] {
		Gate::Zero => { let _ = solver.add_clause(&vec![neg(z)]); },
		Gate::Input(i) => (),
		Gate::Not(x) => {
		    let _ = solver.add_clause(&vec![pos(x),pos(z)]);
		    let _ = solver.add_clause(&vec![neg(x),neg(z)]);
		},
		Gate::Binop(Op::And,x,y) => {
		    let _ = solver.add_clause(&vec![pos(x),pos(y),neg(z)]);
		    let _ = solver.add_clause(&vec![pos(x),neg(y),neg(z)]);
		    let _ = solver.add_clause(&vec![neg(x),pos(y),neg(z)]);
		    let _ = solver.add_clause(&vec![neg(x),neg(y),pos(z)]);
		},
		Gate::Binop(Op::Or,x,y) => {
		    let _ = solver.add_clause(&vec![pos(x),pos(y),neg(z)]);
		    let _ = solver.add_clause(&vec![pos(x),neg(y),pos(z)]);
		    let _ = solver.add_clause(&vec![neg(x),pos(y),pos(z)]);
		    let _ = solver.add_clause(&vec![neg(x),neg(y),pos(z)]);
		},
		Gate::Binop(Op::Xor,x,y) => {
		    if use_xor {
			// z = x ^ y
			// x ^ y ^ z = 0
			let _ = solver.add_xor_literal_clause(&vec![pos(x),pos(y),pos(z)],false);
		    } else {
			let _ = solver.add_clause(&vec![pos(x),pos(y),neg(z)]);
			let _ = solver.add_clause(&vec![pos(x),neg(y),pos(z)]);
			let _ = solver.add_clause(&vec![neg(x),pos(y),pos(z)]);
			let _ = solver.add_clause(&vec![neg(x),neg(y),neg(z)]);
		    }
		}
	    }
	}
	for &(i,b) in constraints.iter() {
	    let _ = solver.add_clause(&vec![if b { pos(i) } else { neg(i) }]);
	}
	solver
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
	let use_xor = true;
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
		    if use_xor {
			write!(fd,"x{} {} {} 0\n",pos(x),pos(y),neg(z))?;
		    } else {
			write!(fd,"{} {} {} 0\n",pos(x),pos(y),neg(z))?;
			write!(fd,"{} {} {} 0\n",pos(x),neg(y),pos(z))?;
			write!(fd,"{} {} {} 0\n",neg(x),pos(y),pos(z))?;
			write!(fd,"{} {} {} 0\n",neg(x),neg(y),neg(z))?;
		    }
		}
	    }
	}
	for &(i,b) in constraints.iter() {
	    write!(fd,"{} 0\n",if b { pos(i) } else { neg(i) })?;
	}
	Ok(())
    }

    fn find(&self,b:&Gate)->Option<Index> {
	self.index.borrow().get(b).map(|x| *x)
    }
    // commutation - canonicalization

    fn get(&self,b:&Gate)->Index {
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

impl GateSoup for Machine {
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
    fn dump(&self,path:&str)->Result<(),std::io::Error> {
	use std::io::Write;
	let fd = std::fs::File::create(path)?;
	let mut fd = std::io::BufWriter::new(fd);
	let spec = self.spec.borrow();
	for i in 0..spec.len() {
	    write!(fd,"x{} <- ",i)?;
	    let v = &spec[i];
	    match v {
		Gate::Zero => writeln!(fd,"0")?,
		Gate::Input(i) => writeln!(fd,"INPUT({})",i)?,
		Gate::Not(i) => writeln!(fd,"!x{}",i)?,
		Gate::Binop(Op::And,i,j) => writeln!(fd,"x{} & x{}",i,j)?,
		Gate::Binop(Op::Or,i,j) => writeln!(fd,"x{} | x{}",i,j)?,
		Gate::Binop(Op::Xor,i,j) => writeln!(fd,"x{} ^ x{}",i,j)?
	    }
	}
	Ok(())
    }
    fn num_inputs(&self)->usize {
	self.n_input.get() as usize
    }
    fn new_input(&self)->Index {
	let i = self.n_input.get();
	self.n_input.set(i + 1);
	self.get(&Gate::Input(i))
    }
    fn input(&self,i:InputIndex)->Index {
	self.get(&Gate::Input(i))
    }
    fn binop(&self,op:Op,a:Index,b:Index)->Index {
	let (a,b) = (a.min(b),a.max(b));
	self.get(&Gate::Binop(op,a,b))
    }
    fn and(&self,a:Index,b:Index)->Index {
	if a == b {
	    a
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    self.get(&Gate::Binop(Op::And,a,b))
	}
    }
    fn or(&self,a:Index,b:Index)->Index {
	if a == b {
	    a
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    self.get(&Gate::Binop(Op::Or,a,b))
	}
    }
    fn xor(&self,a:Index,b:Index)->Index {
	if a == b {
	    self.zero()
	} else {
	    let (a,b) = (a.min(b),a.max(b));
	    self.get(&Gate::Binop(Op::Xor,a,b))
	}
    }
    fn not(&self,a:Index)->Index {
	self.get(&Gate::Not(a))
    }
    fn zero(&self)->Index {
	self.get(&Gate::Zero)
    }
    fn one(&self)->Index {
	self.not(self.zero())
    }
    fn as_input(&self,i:Index)->Option<InputIndex> {
	match self.spec.borrow()[i as usize] {
	    Gate::Input(j) => Some(j),
	    _ => None
	}
    }
}
