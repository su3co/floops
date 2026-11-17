//! SplitMix64 arithmetic shared by CPU and CUDA Oxide event streams.
#[inline]
pub fn next_u64(state: &mut u64) -> u64 {
    *state = (*state).wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
