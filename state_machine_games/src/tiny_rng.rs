use rand::{RngCore, SeedableRng, rand_core::impls};

#[derive(Debug, Clone, PartialEq)]
pub struct TinyRng{
    pub state: u64
}

impl RngCore for TinyRng{
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let ret = self.state;
        self.state = self.state.wrapping_mul(1664525u64).wrapping_add(1013904223u64);
        ret        
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        impls::fill_bytes_via_next(self, dst);
    }
}

impl SeedableRng for TinyRng{
    type Seed = [u8;8];

    fn from_seed(seed: Self::Seed) -> Self {
        
        Self { state: u64::from_le_bytes(seed) }
    }
}

#[cfg(test)]
pub mod tests{
    use rand::{SeedableRng, seq::SliceRandom};

    use crate::tiny_rng::TinyRng;

    #[test]
    pub fn test_rng(){
        let mut rng = TinyRng::seed_from_u64(1234);

        const ARR_LEN: usize = 42;
        const C : usize = 100;

        let mut counts = [[0usize; ARR_LEN];ARR_LEN];
        
        

        for _ in 0..(ARR_LEN * C){
            let mut arr: [usize;ARR_LEN] = std::array::from_fn(|x|x as usize);

            arr.shuffle(&mut rng);

            for (index, x) in arr.iter().enumerate(){
                counts[index][*x as usize] += 1;
            }            
        }

        let mut chi_squared: f64 = 0f64;
        //Pearson's chi squared
        for x in counts.iter().flatten().copied(){
            chi_squared += (x.abs_diff(C) * x.abs_diff(C)) as f64 / (C as f64);
        }

        
        
        insta::assert_debug_snapshot!(counts);
        assert!(chi_squared < 1700.0, "Chi Squared is {chi_squared}");


    }
}