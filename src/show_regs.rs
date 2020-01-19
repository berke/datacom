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
					    val.insert((-k) as u32,false);
					} else {
					    val.insert(k as u32,true);
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
	let n = bits.len();
	for (i,j) in bits.iter() {
	    if val[j] {
		q |= 1_u64 << i;
	    }
	}
	println!("{}[0..{}] : {:08X} {:032b} -- {:02}",reg,n,q,q,q.count_ones());
    }
    Ok(())

    // let reg_path = std::env::args().nth(2).unwrap();
    // let reg_fd = std::fs::File::open(sol_path).unwrap();
    // let mut reg_fd = std::io::BufReader::new(sol_fd);
}
