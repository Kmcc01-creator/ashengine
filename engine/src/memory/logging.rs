use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct MemoryLogger {
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
    total_allocated: AtomicU64,
    total_freed: AtomicU64,
    peak_usage: AtomicU64,
    start_time: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryLogStats {
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub total_allocated: u64,
    pub total_freed: u64,
    pub peak_usage: u64,
    pub uptime: Duration,
}

impl MemoryLogger {
    pub fn new() -> Self {
        info!("Memory logger initialized");
        Self {
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
            total_allocated: AtomicU64::new(0),
            total_freed: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn log_allocation(&self, size: u64, memory_type: u32) {
        let count = self.allocation_count.fetch_add(1, Ordering::SeqCst);
        let total = self.total_allocated.fetch_add(size, Ordering::SeqCst);
        let current_usage = total + size - self.total_freed.load(Ordering::SeqCst);

        // Update peak usage if necessary
        let mut peak = self.peak_usage.load(Ordering::SeqCst);
        while current_usage > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                current_usage,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }

        debug!(
            "Memory allocated: size={}, type={}, total_allocs={}, current_usage={}",
            size,
            memory_type,
            count + 1,
            current_usage
        );
    }

    pub fn log_deallocation(&self, size: u64, memory_type: u32) {
        let count = self.deallocation_count.fetch_add(1, Ordering::SeqCst);
        let freed = self.total_freed.fetch_add(size, Ordering::SeqCst);
        let current_usage = self.total_allocated.load(Ordering::SeqCst) - (freed + size);

        debug!(
            "Memory freed: size={}, type={}, total_deallocs={}, current_usage={}",
            size,
            memory_type,
            count + 1,
            current_usage
        );
    }

    pub fn warn_leak(&self, size: u64, memory_type: u32) {
        warn!(
            "Potential memory leak detected: size={}, type={}",
            size, memory_type
        );
    }

    pub fn log_error(&self, message: &str) {
        error!("Memory error: {}", message);
    }

    pub fn get_stats(&self) -> MemoryLogStats {
        MemoryLogStats {
            allocation_count: self.allocation_count.load(Ordering::SeqCst),
            deallocation_count: self.deallocation_count.load(Ordering::SeqCst),
            total_allocated: self.total_allocated.load(Ordering::SeqCst),
            total_freed: self.total_freed.load(Ordering::SeqCst),
            peak_usage: self.peak_usage.load(Ordering::SeqCst),
            uptime: self.start_time.elapsed(),
        }
    }

    pub fn print_summary(&self) {
        let stats = self.get_stats();
        let current_usage = stats.total_allocated - stats.total_freed;

        info!("Memory Logger Summary:");
        info!("  Uptime: {:?}", stats.uptime);
        info!("  Total allocations: {}", stats.allocation_count);
        info!("  Total deallocations: {}", stats.deallocation_count);
        info!("  Total bytes allocated: {}", stats.total_allocated);
        info!("  Total bytes freed: {}", stats.total_freed);
        info!("  Current memory usage: {}", current_usage);
        info!("  Peak memory usage: {}", stats.peak_usage);

        if current_usage > 0 {
            warn!("  Potential memory leak: {} bytes not freed", current_usage);
        }
    }
}
