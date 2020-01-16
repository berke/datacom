use std::collections::BTreeMap;

pub type Index = u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gate {
    Input(u16),
    Binop(Op,Index,Index)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Op {
    And = 0,
    Or = 1,
    Xor = 2
}

pub struct Machine {
    spec:Vec<Gate>,
    n:usize,
    index:BTreeMap<Gate,Index>
}

impl Machine {
    pub fn dump(&self) {
	for i in 0..self.n {
	    print!("x{} <- ",i);
	    let v = &self.spec[i];
	    match v {
		Gate::Input(i) => println!("INPUT({})",i),
		Gate::Binop(Op::And,i,j) => println!("x{} & x{}",i,j),
		Gate::Binop(Op::Or,i,j) => println!("x{} | x{}",i,j),
		Gate::Binop(Op::Xor,i,j) => println!("x{} ^ x{}",i,j)
	    }
	}
    }
    pub fn new()->Self {
	Machine{
	    spec:Vec::new(),
	    n:0,
	    index:BTreeMap::new()
	}
    }
    pub fn find(&self,b:&Gate)->Option<Index> {
	self.index.get(b).map(|x| *x)
    }
    // commutation - canonicalization

    pub fn get(&mut self,b:&Gate)->Index {
	match self.find(b) {
	    Some(i) => i,
	    None => {
		let i = self.n as u16;
		self.n += 1;
		self.spec.push(*b);
		self.index.insert(*b,i);
		i
	    }
	}
    }
}
