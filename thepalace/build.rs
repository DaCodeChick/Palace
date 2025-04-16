use bytemuck::cast_slice;
use std::fs;
use std::io;

const A: i32 = 16807;
const M: i32 = 2147483647;
const Q: i32 = 127773;
const R: i32 = 2836;

fn random(seed: &mut i32) -> i16 {
    let hi = *seed / Q;
    let lo = *seed % Q;
    let test = A * lo - R * hi;

    *seed = if test > 0 { test } else { test + M };

    (((*seed as f64) / (M as f64)) * 256.0) as i16
}

fn main() -> io::Result<()> {
    let mut table = [0i16; 512];
    let mut seed = 666666i32;

    table.iter_mut().for_each(|x| *x = random(&mut seed));
    fs::write("encode.tbl", cast_slice(&table))?;

    Ok(())
}
