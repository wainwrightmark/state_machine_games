use rand::{Rng, SeedableRng, TryRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TinyRng {
    pub state: u64,
}

impl Default for TinyRng {
    fn default() -> Self {
        Self { state: 12345 }
    }
}

impl TryRng for TinyRng {
    type Error = core::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.next_u64() as u32)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let ret = self.state;
        self.state = self
            .state
            .wrapping_mul(1664525u64)
            .wrapping_add(1013904223u64);
        Ok(ret)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        rand::rand_core::utils::fill_bytes_via_next_word(dst, move || self.try_next_u64())
    }
}

impl SeedableRng for TinyRng {
    type Seed = [u8; 8];

    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            state: u64::from_le_bytes(seed),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use rand::{SeedableRng, seq::SliceRandom};

    use crate::tiny_rng::TinyRng;

    #[test]
    pub fn test_rng() {
        let mut rng = TinyRng::seed_from_u64(1234);

        const ARR_LEN: usize = 42;
        const C: usize = 100;

        let mut counts = [[0usize; ARR_LEN]; ARR_LEN];

        for _ in 0..(ARR_LEN * C) {
            let mut arr: [usize; ARR_LEN] = std::array::from_fn(|x| x as usize);

            arr.shuffle(&mut rng);

            for (index, x) in arr.iter().enumerate() {
                counts[index][*x as usize] += 1;
            }
        }

        let mut chi_squared: f64 = 0f64;
        //Pearson's chi squared
        for x in counts.iter().flatten().copied() {
            chi_squared += (x.abs_diff(C) * x.abs_diff(C)) as f64 / (C as f64);
        }

        insta::assert_debug_snapshot!(counts);
        assert!(chi_squared < 1700.0, "Chi Squared is {chi_squared}");
    }
}
