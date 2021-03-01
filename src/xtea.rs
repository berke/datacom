use crate::utils::*;

pub fn encipher((x0,x1):(u32,u32), k:[u32;4],nround:usize)->(u32,u32)
{
    let mut sum : u32 = 0;
    let delta = 0x9e3779b9;

    let mut v0 = x0;
    let mut v1 = x1;

    for _ in 0..nround {
	// println!("ROUND {:08X} {:08X} {:08X}",sum,v0,v1);
	v0 = v0.wrapping_add((((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ (sum.wrapping_add(k[(sum & 3) as usize])));
        sum = sum.wrapping_add(delta);
        v1 = v1.wrapping_add((((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)) ^ (sum.wrapping_add(k[((sum>>11) & 3) as usize])));
    }
    (v0,v1)
}

pub fn encipher1(x:u64,k:u128,nround:usize)->u64 {
    let (y0,y1) = encipher(sp(x),sp128(k),nround);
    jn(y0,y1)
}
