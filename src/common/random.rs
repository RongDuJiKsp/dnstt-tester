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
    #[allow(dead_code)]
    pub fn random_bytes(&mut self) -> Vec<u8> {
        let size = self.rand_maker.gen_range(self.min_size..=self.max_size);
        let mut buf = vec![0u8; size as usize];
        self.rand_maker.fill_bytes(&mut buf);
        buf
    }
    #[allow(dead_code)]
    pub fn random_printable_line(&mut self) -> Vec<u8> {
        let size = self.rand_maker.gen_range(self.min_size..=self.max_size);
        let print_st = 32u8;
        let print_ed = 126u8;
        let mut r = (0..size)
            .map(|_| {
                let random_char = rand::random::<u8>();
                print_st + (random_char % (print_ed - print_st + 1))
            })
            .collect();
        Vec::push(&mut r, b'\n');
        r
    }
}
