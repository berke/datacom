pub fn encipher((x0,x1):(u32,u32), k:[u32;4])->(u32,u32)
{
    let mut sum : u32 = 0;
    let delta = 0x9e3779b9;

    let mut v0 = x0;
    let mut v1 = x1;

    for _ in 0..32 {
        v0 += (((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ (sum.wrapping_add(k[(sum & 3) as usize]));
        sum += delta;
        v1 += (((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)) ^ (sum.wrapping_add(k[((sum>>11) & 3) as usize]));
    }
    (v0,v1)
}