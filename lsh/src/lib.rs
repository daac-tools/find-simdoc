pub mod minhash;
pub mod simhash;

#[inline(always)]
pub fn hash_u64(x: u64, seed: u64) -> u64 {
    fasthash::city::hash64_with_seed(x.to_le_bytes(), seed)
}
