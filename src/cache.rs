pub(crate) fn make_layout_key(
    ctx: &crate::engine::InternalLayoutContext,
    engine: &crate::LayoutEngine,
) -> u32 {
    use std::hash::{Hash, Hasher};

    let mut h = self::LayoutHasher::default();

    ctx.containing_block_width.map(f32::to_bits).hash(&mut h);
    ctx.containing_block_height.map(f32::to_bits).hash(&mut h);
    f32::to_bits(engine.viewport_width).hash(&mut h);
    f32::to_bits(engine.viewport_height).hash(&mut h);

    h.finish() as u32
}

use std::hash::Hasher;

/// Lightweight hasher for layout caching.
/// Designed for hashing layout-related data efficiently using FNV-like algorithm.
#[derive(Debug, Clone)]
struct LayoutHasher {
    state: u64,
}

impl LayoutHasher {
    #[inline]
    pub fn new() -> Self {
        // FNV offset basis (64-bit)
        Self {
            state: 0xcbf29ce484222325,
        }
    }

    #[inline]
    fn mix(&mut self, v: u64) {
        // XOR followed by multiplication with golden ratio for avalanche effect
        self.state ^= v;
        self.state = self.state.wrapping_mul(0x9E3779B97F4A7C15);
    }
}

impl Default for LayoutHasher {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for LayoutHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // Process bytes in 8-byte chunks for layout data hashing
        let mut i = 0;
        while i + 8 <= bytes.len() {
            let chunk = u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            self.mix(chunk);
            i += 8;
        }

        // Handle remaining bytes
        if i < bytes.len() {
            let mut last = 0u64;
            for (shift, b) in bytes[i..].iter().enumerate() {
                last |= (*b as u64) << (shift * 8);
            }
            self.mix(last);
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.mix(i);
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.mix(i as u64);
    }
}
