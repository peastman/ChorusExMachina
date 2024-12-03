use std::time::{SystemTime, UNIX_EPOCH};

const UNIFORM_SCALE: f32 = 1.0/(0x100000000i64 as f32);

/// This is a quick and dirty random number generator.  It is based on the
/// "even quicker generator" in Numerical Recipes.  Its statistical properties
/// aren't great, but it's fine for our purposes, and it's very fast.  It also
/// precalculates a list of normally distributed values.  When you ask for a
/// normal value, it simply returns a randomly chosen one from the list.
pub struct Random {
    i: u32,
    normal_table: [f32;1024]
}

impl Random {
    pub fn new() -> Self {
        // Select a seed.

        let time = SystemTime::now().duration_since(UNIX_EPOCH);
        let seed = match time {
            Ok(t) => t.subsec_nanos(),
            Err(_) => 0
        };

        // Precalculate some normally distributed values.

        let mut rand = Self {i: seed, normal_table: [0.0; 1024]};
        let mut i = 0;
        while i < 1024 {
            let x = 2.0*rand.get_uniform()-1.0;
            let y = 2.0*rand.get_uniform()-1.0;
            let r2 = x*x + y*y;
            if r2 < 1.0 && r2 != 0.0 {
                let multiplier = (-2.0*r2.ln()/r2).sqrt();
                rand.normal_table[i] = x*multiplier;
                rand.normal_table[i+1] = y*multiplier;
                i += 2;
            }
        }
        let shift = rand.normal_table.iter().sum::<f32>() / 1024.0;
        for i in 0..1024 {
            rand.normal_table[i] -= shift;
        }
        rand
    }

    /// Get a random integer.
    pub fn get_int(&mut self) -> u32 {
        self.i = ((self.i as u64)*1664525u64 + 1013904223u64) as u32;
        self.i
    }

    /// Get a random value, uniformly distributed between 0.0 and 1.0.
    pub fn get_uniform(&mut self) -> f32 {
        UNIFORM_SCALE * (self.get_int() as f32)
    }

    /// Get a random value from a normal distribution.
    pub fn get_normal(&mut self) -> f32 {
        self.normal_table[self.get_int() as usize % 1024]
    }
}
