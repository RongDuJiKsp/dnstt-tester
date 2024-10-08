use rand::rngs::ThreadRng;
use rand::{Rng, RngCore};

pub struct RandomPacker {
    min_size: u64,
    max_size: u64,
    rand_maker: ThreadRng,
}
impl RandomPacker {
    pub fn new(min_size: u64, max_size: u64) -> RandomPacker {
        let rand_maker = rand::thread_rng();
        RandomPacker {
            max_size,
            min_size,
            rand_maker,
        }
    }
    pub fn random_bytes(&mut self) -> Vec<u8> {
        let size = self.rand_maker.gen_range(self.min_size..=self.max_size);
        let mut buf = vec![0u8; size as usize];
        self.rand_maker.fill_bytes(&mut buf);
        buf
    }
}
