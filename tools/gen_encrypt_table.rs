/// Generates the Palace encryption table from seed 666666
///
/// This tool generates the 512-byte encryption lookup table used by
/// Palace's XOR cipher. The table is generated using the Park-Miller
/// PRNG algorithm with a specific seed value.
///
/// Random number generator algorithm:
/// Stephen K. Park and Keith W. Miller,
/// "Random Number Generators: Good Ones Are Hard to Find",
/// Communications of the ACM, vol. 31, p. 1192 (October 1988).

const R_A: i32 = 16807;
const R_M: i32 = 2147483647;
const R_Q: i32 = 127773;
const R_R: i32 = 2836;

struct Random {
    seed: i32,
}

impl Random {
    fn new(seed: i32) -> Self {
        let seed = if seed == 0 { 1 } else { seed };
        Self { seed }
    }

    fn long_random(&mut self) -> i32 {
        let hi = self.seed / R_Q;
        let lo = self.seed % R_Q;
        let test = R_A * lo - R_R * hi;
        self.seed = if test > 0 { test } else { test + R_M };
        self.seed
    }

    fn double_random(&mut self) -> f64 {
        self.long_random() as f64 / R_M as f64
    }

    fn my_random(&mut self, max: i16) -> i16 {
        (self.double_random() * max as f64) as i16
    }
}

fn main() {
    let mut rng = Random::new(666666);

    println!("// Palace encryption lookup table");
    println!("// Generated from seed 666666 using Park-Miller PRNG");
    println!("// 512 bytes: values 0-255");
    println!("pub const ENCRYPT_TABLE: [u8; 512] = [");

    for i in 0..512 {
        if i % 16 == 0 {
            print!("    ");
        }

        let value = rng.my_random(256) as u8;
        print!("{:#04x}", value);

        if i < 511 {
            print!(", ");
        }

        if (i + 1) % 16 == 0 {
            println!();
        }
    }

    println!("\n];");
}
