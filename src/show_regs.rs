use std::collections::BTreeMap;
use std::io::{BufRead,Read,Write};

fn load_regs(path:&str)->Result<BTreeMap<String,BTreeMap<u32,u32>>,Box<dyn std::error::Error>> {
    let fd = std::fs::File::open(path)?;
    let mut fd = std::io::BufReader::new(fd);
    let mut buf = String::new();
    let mut regs = BTreeMap::new();
    loop {
	buf.clear();
	match fd.read_line(&mut buf) {
	    Ok(0) | Ok(1) => break,
	    Ok(n) => {
		let u = &mut buf[0..n-1].chars();
		let vs : Vec<&str> = u.as_str().split(|c| c == ' ' || c == '\n').collect();
		if vs.len() != 3 {
		    println!("Warning: Bad register description");
		} else {
		    let reg = String::from(vs[0]);
		    let i = vs[1].parse::<u32>()?;
		    let j = vs[2].parse::<u32>()?;
		    match regs.get_mut(&reg) {
			None => {
			    let mut btr = BTreeMap::new();
			    btr.insert(i,j);
			    regs.insert(reg,btr);
			},
			Some(btr) => {
			    btr.insert(i,j);
			}
		    }
		}
	    },
	    Err(e) => return Err(Box::new(e))
	}
    }
    Ok(regs)
}

fn load_valuation(path:&str)->Result<BTreeMap<u32,bool>,std::io::Error> {
    let sol_fd = std::fs::File::open(path)?;
    let mut sol_fd = std::io::BufReader::new(sol_fd);
    let mut buf = String::new();
    let mut val = BTreeMap::new();
    loop {
	buf.clear();
	match sol_fd.read_line(&mut buf) {
	    Ok(0) | Ok(1) => break,
	    Ok(n) => {
		let u = &mut buf[0..n-1].chars();
		match u.next() {
		    Some('v') => {
			let vs : Vec<&str> = u.as_str().split(|c| c == ' ' || c == '\n').collect();
			for k in vs.iter() {
			    if k.len() > 0 {
				match k.parse::<i32>() {
				    Err(_) => println!("Warning: Bad integer {:?}",k),
				    Ok(k) => {
					if k < 0 {
					    val.insert((-k-1) as u32,false);
					} else {
					    val.insert((k-1) as u32,true);
					}
				    }
				}
			    }
			}
		    },
		    _ => ()
		}
	    },
	    Err(e) => return Err(e)
	}
    }
    Ok(val)
}

fn main()->Result<(),Box<dyn std::error::Error>> {
    let sol_path = std::env::args().nth(1).unwrap();
    let reg_path = std::env::args().nth(2).unwrap();
    let val = load_valuation(&sol_path)?;
    let regs = load_regs(&reg_path)?;
    for (reg,bits) in regs.iter() {
	let mut q : u64 = 0;
	let n = bits.len() as u32;
	if n <= 64 {
	    for (i,j) in bits.iter() {
		match val.get(j) {
		    Some(true) => q |= 1_u64 << i,
		    Some(false) => (),
		    None => panic!("Register {} bit {} gate {} undefined",reg,i,j)
		}
	    }
	    println!("{}[{}..0] : {:0w1$X} {:0w2$b} | {:w3$} -- {:3}",
		     reg,n,q,q,q,q.count_ones(),
		     w1=((n+3)/4) as usize,
		     w2=n as usize,
		     w3=((10*n+32)/33) as usize);
	} else {
	    let mut nl = true;
	    let c = 32;
	    for k in 0..n {
		let i = n - 1 - k;
		if k % 32 == 0 {
		    if k > 0 {
			println!("");
		    }
		    print!("{}[{:02X}..] ",reg,i);
		}
		let j = bits.get(&i).unwrap();
		print!("{}", if val[j] { 1 } else { 0 });
	    }
	    println!("");
	}
    }
    Ok(())
}
