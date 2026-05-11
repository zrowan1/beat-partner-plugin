use atomic_float::AtomicF32;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A lock-free single-producer single-consumer ring buffer for real-time audio.
///
/// # Real-time Safety
/// - `write()` is wait-free and lock-free, safe to call from the audio thread.
/// - `read()` is wait-free and lock-free, safe to call from the background thread.
/// - No allocations occur after construction.
pub struct LockFreeRingBuffer {
    buffer: Vec<AtomicF32>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}

impl LockFreeRingBuffer {
    /// Create a new ring buffer with the given capacity.
    /// Capacity is rounded up to the next power of two for efficient masking.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(64);
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(AtomicF32::new(0.0));
        }
        Self {
            buffer,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            capacity,
        }
    }

    /// Write interleaved samples into the ring buffer.
    /// Call this from the real-time audio thread.
    pub fn write(&self, samples: &[f32]) {
        let mut write_pos = self.write_pos.load(Ordering::Relaxed);
        for &sample in samples {
            let idx = write_pos & (self.capacity - 1);
            self.buffer[idx].store(sample, Ordering::Relaxed);
            write_pos = write_pos.wrapping_add(1);
        }

        // Advance read_pos if the writer has lapped the reader.
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let written = write_pos.wrapping_sub(read_pos);
        if written > self.capacity {
            self.read_pos
                .store(write_pos.wrapping_sub(self.capacity), Ordering::Relaxed);
        }

        self.write_pos.store(write_pos, Ordering::Release);
    }

    /// Read samples from the ring buffer into `out`.
    /// Returns the number of samples actually read.
    /// Call this from the background analysis thread.
    pub fn read(&self, out: &mut [f32]) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let available = write_pos.wrapping_sub(read_pos);
        let to_read = out.len().min(available);

        let mut pos = read_pos;
        for i in 0..to_read {
            let idx = pos & (self.capacity - 1);
            out[i] = self.buffer[idx].load(Ordering::Relaxed);
            pos = pos.wrapping_add(1);
        }

        self.read_pos.store(pos, Ordering::Relaxed);
        to_read
    }

    /// Return how many samples are currently available to read.
    pub fn available(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        write_pos.wrapping_sub(read_pos)
    }

    /// Reset the buffer, clearing all contents.
    pub fn clear(&self) {
        self.read_pos
            .store(self.write_pos.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_write_read() {
        let rb = LockFreeRingBuffer::new(16);
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        rb.write(&samples);

        let mut out = [0.0; 4];
        let read = rb.read(&mut out);
        assert_eq!(read, 4);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_wraparound() {
        // Capacity is rounded up to at least 64.
        let rb = LockFreeRingBuffer::new(64);
        // Write 80 samples to force wraparound
        let samples: Vec<f32> = (0..80).map(|i| i as f32).collect();
        rb.write(&samples);

        let mut out = [0.0; 80];
        let read = rb.read(&mut out);
        // Only the last 64 samples are retained due to overwrite
        assert_eq!(read, 64);
    }

    #[test]
    fn test_available() {
        let rb = LockFreeRingBuffer::new(16);
        assert_eq!(rb.available(), 0);
        rb.write(&[1.0, 2.0]);
        assert_eq!(rb.available(), 2);
    }
}
