fn main() {
    let mut mac = Machine::new();
    // let mut mac = Bracket::new();
    let mut in_constraints : Vec<(Index,bool)> = Vec::new();
    let mut out_constraints : Vec<(Index,bool)> = Vec::new();
    let mut r1 = Register::input(&mut mac,19);   
    let mut r2 = Register::input(&mut mac,22);
    let mut r3 = Register::input(&mut mac,23);
    let mut r4 = Register::input(&mut mac,17);
    // constraints.append(&mut r4.constraints(0xdeadbeef));

    //let mut xw = Xorwow::new(129837471234567);
    let mut xw = Xorwow::new(19934559142);
    let helper = !0;

    let k1 = xw.next() & ((1 << 19) - 1);
    let k2 = xw.next() & ((1 << 22) - 1);
    let k3 = xw.next() & ((1 << 23) - 1);
    let k4 = xw.next() & ((1 << 17) - 1);
    println!("k1 = {:08X}",k1);
    println!("k2 = {:08X}",k2);
    println!("k3 = {:08X}",k3);
    println!("k4 = {:08X}",k4);

    in_constraints.append(&mut r1.constraints(k1 as u64));
    in_constraints.append(&mut r2.constraints(k2 as u64));
    in_constraints.append(&mut r3.constraints(k3 as u64));
    in_constraints.append(&mut r4.constraints(k4 as u64));

    let r1_c = r1.clone();
    let r2_c = r2.clone();
    let r3_c = r3.clone();
    let r4_c = r4.clone();
    let mut reg_info =
		   vec![
		       ("r1".to_string(),&r1_c),
		       ("r2".to_string(),&r2_c),
		       ("r3".to_string(),&r3_c),
		       ("r4".to_string(),&r4_c)];

    // let mut r4 : u32 = xw.next();
    let bl = |x| if x { 1 } else { 0 };

    // (a&b)|(a&c)|(b&c)
    // m = maj(a,b,c)
    // a b c m w g
    // 0 0 0 0 0 1
    // 0 0 0 1 0 0
    // 0 0 1 0 0 1 
    // 0 0 1 1 0 0
    // 0 1 0 0 0 0
    // 0 1 0 1 0 0
    // 0 1 1 0 1 0
    // 0 1 1 1 1 1
    // 1 0 0 0 0 1
    // 1 0 0 1 0 0
    // 1 0 1 0 1 0
    // 1 0 1 1 1 1
    // 1 1 0 0 1 0
    // 1 1 0 1 1 1
    // 1 1 1 0 1 0
    // 1 1 1 1 1 1

    let mut outputs = Vec::new();

    let maj = |a,b,c| mac.or(mac.or(mac.and(a,b),mac.and(a,c)),
			     mac.and(b,c));

    let t_start = now();
    
    for t in 0..2*81 {
	let f = mac.xor(r4.bit(16),r4.bit(11));
	r4 = r4.rotate_left(1);
	r4.set_bit(0,f);

	let a = r4.bit(3);
	let b = r4.bit(7);
	let c = r4.bit(10);

	let mj = maj(a,b,c);
	let eq = |x,y| mac.or(mac.and(mac.not(x),mac.not(y)),mac.and(x,y));
	let c1 = eq(c,mj);
	let c2 = eq(a,mj);
	let c3 = eq(b,mj);

	let f1 = mac.xor(r1.bit(18),
			 mac.xor(r1.bit(17),
				 r1.bit(14)));
	let mut r1_clk = r1.rotate_left(1);
	r1_clk.set_bit(0,f1);
	let r1_clk = r1_clk.scale(&mac,c1);
	r1 = r1.scale(&mac,mac.not(c1)).or(&mac,&r1_clk);

	let f2 = mac.xor(r2.bit(21),
			 r2.bit(20));
	let mut r2_clk = r2.rotate_left(1);
	r2_clk.set_bit(0,f2);
	let r2_clk = r2_clk.scale(&mac,c2);
	r2 = r2.scale(&mac,mac.not(c2)).or(&mac,&r2_clk);

	let f3 =
	    mac.xor(
		mac.xor(r3.bit(22),
			r3.bit(21)),
		r3.bit(7));
	let mut r3_clk = r3.rotate_left(1);
	r3_clk.set_bit(0,f2);
	let r3_clk = r3_clk.scale(&mac,c3);
	r3 = r3.scale(&mac,mac.not(c3)).or(&mac,&r3_clk);

	let m1 = maj(r1.bit(15),mac.not(r1.bit(14)),r1.bit(12));
	let m2 = maj(mac.not(r2.bit(16)),r2.bit(13),r2.bit(9));
	let m3 = maj(r3.bit(18),r3.bit(16),mac.not(r3.bit(13)));

	let o = mac.xor(m1,mac.xor(m2,m3));
	outputs.push(o);
    }

    let mut xw = Xorwow::new(4);
    let v = mac.eval(&in_constraints);
    for &u in outputs.iter() {
	let p = xw.next() as f64 / ((1_u64 << 32) - 1) as f64;
	if p <= 1.00 {
	    out_constraints.push((u,v[u as usize]));
	}
	// println!("OUT {} -> {}",u,v[u as usize]);
    }
    
    let mut xw = Xorwow::new(10);
    // let mut k = 0;
    // for &(u,b) in in_constraints.iter() {
    // 	let p = xw.next() as f64 / ((1_u64 << 32) - 1) as f64;
    // 	if p < 0.10 {
    // 	    // out_constraints.push((u,b));
    // 	    let b = (xw.next() & 1) != 0;
    // 	    out_constraints.push((u,b));
    // 	    k += 1;
    // 	}
    // }
    // println!("Key constraints provided: {}/{}",k,in_constraints.len());

    let mut rnd = || {
	xw.next() as f64 / ((1_u64 << 32) - 1) as f64
    };

    let mut rnd_int = |n:usize| {
	((rnd() * n as f64).floor() as usize).min(n-1)
    };

    let mut key = r1.clone();
    key.append(&mut r2.clone());
    key.append(&mut r3.clone());
    key.append(&mut r4.clone());

    let m = key.len();

    // let mut set = Vec::new();
    // set.resize(m,false);
    // let mut i;
    // for k in 0..38 {
    // 	loop {
    // 	    i = rnd_int(m);
    // 	    if !set[i] {
    // 		break;
    // 	    }
    // 	}
    // 	set[i] = true;
    // 	let b = rnd_int(2) != 0;
    // 	out_constraints.push((key.bit(i),b));
    // }

    // let mut o = mac.zero();
    // for l in 0..100 {
    // 	let mut a = mac.one();
    // 	for k in 0..30 {
    // 	    let i = rnd_int(m - 1);
    // 	    let j = i + rnd_int(m - i - 1);
    // 	    let o = mac.xor(key.bit(i),key.bit(j));
    // 	    a = mac.and(a,mac.not(o));
    // 	}
    // 	o = mac.or(o,a);
    // }
    // out_constraints.push((o,true));
    // for &(u,b) in in_constraints.iter() {
    // 	if p < 0.10 {
    // 	    // out_constraints.push((u,b));
    // 	    let b = (xw.next() & 1) != 0;
    // 	    out_constraints.push((u,b));
    // 	    k += 1;
    // 	}
    // }

    //out_constraints.append(&mut Vec::from(&mut in_constraints[0..79]));
    // out_constraints.append(&mut in_constraints.clone());
    // mac.save_cnf("mac.cnf",&out_constraints).unwrap();
    // mac.save("mac.alg",&out_constraints).unwrap();
    Register::dump(&mac,"mac.reg",reg_info).unwrap();

    let mut solver = mac.solver(&out_constraints);
    // let p = solver.nvars() as usize;
    let p = m;
    println!("Solving...");
    let mut picked = Vec::new();
    let mut selected = Vec::new();
    let mut known = Vec::new();
    let mut values = Vec::new();
    let mut ass = Vec::new();
    picked.resize(p,false);
    known.resize(p,false);
    values.resize(p,false);
    let q = 18;
    let mut i;
    let mut found = 0;
    let mut cnt = 0;
    let mut seen = BTreeSet::new();
    let mut total = 0;
    
    let mut max_time = 0.2;
    let p = 18;

    let mut qe = QuadraticEstimator::new();

    let t_start = now();
    loop {
	if found >= p {
	    break;
	}
	

	loop {
	    // Make some random assumptions
	    ass.clear();
	    for i in 0..m {
		picked[i] = false;
	    }
	    // let p = rnd_int(q) + 1;

	    selected.clear();
	    for k in 0..p {
		loop {
		    i = rnd_int(p);
		    if !picked[i] {
			break;
		    }
		}
		selected.push(i);
		ass.push(Lit::new(key.bit(i),rnd_int(2) != 0).unwrap());
		picked[i] = true;
	    }
	    let mut ass2 = ass.clone();
	    ass2.sort();
	    if !seen.contains(&ass2) {
		seen.insert(ass2);
		break;
	    }
	}

	// let u = key.bit(i);
	// let u = i as u32;
	
	solver.set_max_time(max_time);
	let t0 = now() - t_start;
	let ret = solver.solve_with_assumptions(&ass);
	let t1 = now() - t_start;
	total += 1;
	let dt = t1 - t0;
	// println!("{} {:?}",p,ret);
	// print!("{:.3} ",dt);
	match ret {
	    Lbool::False => {
		println!("F{} in {:.3}/{:.3}",p,dt,max_time);
		max_time = 0.9 * max_time + 0.1 * 1.5 * dt;
		if p == 1 {
		    i = selected[0];
		    let v = !ass[0].isneg();
		    if !known[i] {
			known[i] = true;
			values[i] = v;
			println!("Found bit {} = {}",i,v);
			found += 1;
		    } else {
			if values[i] != v {
			    panic!("Contradiction on bit {}, found {}, was {}",i,v,values[i]);
			}
		    }
		} else {
		    print!("NOT(");
		    for k in 0..p {
			print!(" k{:03}={}",selected[k],if ass[k].isneg() { 1 } else { 0 });
		    }
		    println!(" )");
		}

		let a : Vec<Lit> = ass.iter().map(|&l| !l).collect();
		solver.add_clause(&a);
		cnt += 1;
		qe.push(t1,cnt as f64);
		let rate = (now() - t_start)/cnt as f64;
		println!("CNT {}, APPROX EVERY {} s OR EVERY {} SOLVE, ETA {} h",cnt,rate,
			 total as f64/cnt as f64,
			 rate * (1 << p) as f64 / 3600.0);
		match qe.solve_for_t((1 << p) as f64) {
		    None => (),
		    Some(t) => println!("ETA {} h",t/3600.0)
		}
	    },
	    Lbool::Undef => {
		max_time *= 1.01
		// println!("U{}",p);
	    }
	    Lbool::True => {
		println!("FOUND!");
		break;
	    }
	}
	// solver.set_max_time(max_time);
	// let ret1 = solver.solve_with_assumptions(&ass1);
	// println!("{:3}: {:?} {:?}",i,ret0,ret1);
	// match (ret0,ret1) {
	//     (Lbool::False,Lbool::Undef) | (Lbool::False,Lbool::True) => (),
	//     (Lbool::Undef,Lbool::False) | (Lbool::True,Lbool::False) => (),
	//     | _ => ()
	// };
	// if ret0 == Lbool::False {
	//     println!("Eliminated");
	//     ass.push(Lit::new(u,false).unwrap());
	// } else {
	//     solver.set_max_time(max_time);
	//     let ret1 = solver.solve_with_assumptions(&ass1);
	//     if ret1 == Lbool::False {
	// 	println!("Bit {} must be false",i);
	// 	known[i] = true;
	// 	vals[i] = false;
	// 	found += 1;
	// 	ass.push(Lit::new(u,true).unwrap());
	//     } else {
	// 	println!("Could not determine bit {}",i);
	//     }
	// }


	//     // Check...
	//     println!("ASS1: {:?}",ret);

	//     if ret == Lbool::False {
	// 	println!("Inconsistent");
	//     } else {
	//     }
	// } else {
	// }
    }
    // println!("RECOVERED KEY");
    // println!("-------------");
    // for i in 0..m {
    // 	print!("{}",if known[i] { if vals[i] { '1' } else { '0' } } else { '?' });
    // }
    println!();
}
