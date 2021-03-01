use crate::utils::*;

pub fn encipher((x0,x1):(u32,u32), k:[u32;4],nround:usize)->(u32,u32)
{
    let mut sum : u32 = 0;
    let delta = 0x9e3779b9;

    let mut v0 = x0;
    let mut v1 = x1;

    for _ in 0..nround {
	sum += delta;
	v0 = v0.wrapping_add(
	    (v1 << 4).wrapping_add(k[0]) ^
		v1.wrapping_add(sum) ^
		(v1 >> 5).wrapping_add(k[1]));
	v1 = v1.wrapping_add(
	    (v0 << 4).wrapping_add(k[2]) ^
		v0.wrapping_add(sum) ^
		(v0 >> 5).wrapping_add(k[3]));
    }
    (v0,v1)
}

pub fn encipher1(x:u64,k:u128,nround:usize)->u64 {
    let (y0,y1) = encipher(sp(x),sp128(k),nround);
    jn(y0,y1)
}
