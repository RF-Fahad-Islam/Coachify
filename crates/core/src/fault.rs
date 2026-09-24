use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::RwLock;

static ACTIVE_CHECKPOINT: RwLock<Option<String>> = RwLock::new(None);
static SIMULATED_DISK_FREE: RwLock<Option<u64>> = RwLock::new(None);
static SIMULATED_CLOCK_OFFSET: AtomicI64 = AtomicI64::new(0);
static HAS_CLOCK_OFFSET: AtomicBool = AtomicBool::new(false);
static SIMULATE_USB_REMOVAL: AtomicBool = AtomicBool::new(false);
static SIMULATE_SMS_TIMEOUT: AtomicBool = AtomicBool::new(false);

/// Sets an explicit checkpoint name in memory for the current process.
pub fn set_checkpoint(name: Option<&str>) {
    if let Ok(mut lock) = ACTIVE_CHECKPOINT.write() {
        *lock = name.map(|s| s.to_string());
    }
}

/// Triggers a checkpoint if either `COACHIFY_CRASH_CHECKPOINT` env var or
/// the in-memory checkpoint matches `name`.
pub fn trigger_checkpoint(name: &str) {
    let mut matched = false;

    if let Ok(env_val) = std::env::var("COACHIFY_CRASH_CHECKPOINT") {
        if env_val.trim() == name {
            matched = true;
        }
    }

    if !matched {
        if let Ok(lock) = ACTIVE_CHECKPOINT.read() {
            if let Some(ref active) = *lock {
                if active == name {
                    matched = true;
                }
            }
        }
    }

    if matched {
        println!("CHECKPOINT_REACHED");
        let _ = std::io::stdout().flush();
        // Block indefinitely until terminated by parent process
        let mut buf = [0u8; 1];
        let _ = std::io::stdin().read_exact(&mut buf);
    }
}

/// Configures simulated disk free space in bytes.
pub fn set_simulated_disk_free(bytes: Option<u64>) {
    if let Ok(mut lock) = SIMULATED_DISK_FREE.write() {
        *lock = bytes;
    }
}

/// Returns the simulated disk free space in bytes, if set.
pub fn get_simulated_disk_free() -> Option<u64> {
    if let Ok(lock) = SIMULATED_DISK_FREE.read() {
        *lock
    } else {
        None
    }
}

/// Configures a simulated clock offset in seconds (positive for future, negative for past).
pub fn set_simulated_clock_offset_seconds(secs: Option<i64>) {
    match secs {
        Some(s) => {
            SIMULATED_CLOCK_OFFSET.store(s, Ordering::SeqCst);
            HAS_CLOCK_OFFSET.store(true, Ordering::SeqCst);
        }
        None => {
            SIMULATED_CLOCK_OFFSET.store(0, Ordering::SeqCst);
            HAS_CLOCK_OFFSET.store(false, Ordering::SeqCst);
        }
    }
}

/// Returns the simulated clock offset in seconds, if active.
pub fn get_simulated_clock_offset_seconds() -> Option<i64> {
    if HAS_CLOCK_OFFSET.load(Ordering::SeqCst) {
        Some(SIMULATED_CLOCK_OFFSET.load(Ordering::SeqCst))
    } else {
        None
    }
}

/// Sets whether USB removal is simulated during backup writes.
pub fn set_simulate_usb_removal(active: bool) {
    SIMULATE_USB_REMOVAL.store(active, Ordering::SeqCst);
}

/// Returns whether USB removal is simulated.
pub fn is_usb_removal_simulated() -> bool {
    SIMULATE_USB_REMOVAL.load(Ordering::SeqCst)
}

/// Sets whether SMS delivery timeout is simulated.
pub fn set_simulate_sms_timeout(active: bool) {
    SIMULATE_SMS_TIMEOUT.store(active, Ordering::SeqCst);
}

/// Returns whether SMS delivery timeout is simulated.
pub fn is_sms_timeout_simulated() -> bool {
    SIMULATE_SMS_TIMEOUT.load(Ordering::SeqCst)
}

/// Resets all simulated environmental and peripheral faults to normal.
pub fn reset_all_faults() {
    set_checkpoint(None);
    set_simulated_disk_free(None);
    set_simulated_clock_offset_seconds(None);
    set_simulate_usb_removal(false);
    set_simulate_sms_timeout(false);
}
