struct QuadraticEstimator {
    t:[f64;3],
    f:[f64;3],
    n:usize,
    i:usize
}

impl QuadraticEstimator {
    pub fn new()->Self {
	QuadraticEstimator{
	    t:[0.0;3],
	    f:[0.0;3],
	    n:0,
	    i:0
	}
    }
    pub fn push(&mut self,t:f64,f:f64) {
	self.t[self.i] = t;
	self.f[self.i] = f;
	self.i += 1;
	if self.i == 3 {
	    self.i = 0;
	}
	if self.n < 3 {
	    self.n += 1;
	}
    }
    pub fn solve_for_t(&self,f:f64)->Option<f64> {
	if self.n < 3 {
	    return None;
	}
	let i1 = self.i;
	let i2 = (i1 + 1) % 3;
	let i3 = (i1 + 2) % 3;
	let m1 = self.f[i1];
	let m2 = self.f[i2];
	let m3 = self.f[i3];
	let t1 = self.t[i1];
	let t2 = self.t[i2];
	let t3 = self.t[i3];
	println!("m1={} m2={} m3={} t1={} t2={} t3={}",m1,m2,m3,t1,t2,t3);
	let d = ((t2-t1)*t3*t3+(t1*t1-t2*t2)*t3+t1*t2*t2-t1*t1*t2);
	let a = ((m1*t2-m2*t1)*t3*t3+(m2*t1*t1-m1*t2*t2)*t3+m3*t1*t2*t2-m3*t1*t1*t2) / d;
	let b = ((m2-m1)*t3*t3+(m1-m3)*t2*t2+(m3-m2)*t1*t1) / d;
	let c = -((m2-m1)*t3+(m1-m3)*t2+(m3-m2)*t1) / d;
	println!("a={} b={} c={}",a,b,c);
	let dt = 4.0*c*f-4.0*a*c+b*b;
	if dt < 0.0 {
	    None
	} else {
	    Some((dt.sqrt()-b)/(2.0*c))
	}
    }
}
