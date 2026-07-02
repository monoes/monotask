use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};

static LOGICAL: AtomicU64 = AtomicU64::new(0);

/// Returns an HLC timestamp string: "<wall_ms>-<logical>".
pub fn now() -> String {
    let wall = Utc::now().timestamp_millis().max(0) as u64;
    let logical = LOGICAL.fetch_add(1, Ordering::SeqCst);
    format!("{wall:016x}-{logical:08x}")
}

/// Advance the logical counter from an incoming remote HLC timestamp.
/// Call this when receiving any event from a remote peer so local `now()` stays
/// causally after all observed remote events.
pub fn observe_remote(remote_hlc: &str) {
    if let Some(logical_part) = remote_hlc.split('-').nth(1) {
        if let Ok(remote_logical) = u64::from_str_radix(logical_part, 16) {
            let mut cur = LOGICAL.load(Ordering::SeqCst);
            loop {
                if remote_logical < cur {
                    break;
                }
                match LOGICAL.compare_exchange(cur, remote_logical + 1, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(actual) => cur = actual,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_remote_advances_logical() {
        // Manufacture a remote HLC with a large logical counter
        let remote = format!("0000000000000000-{:08x}", 0xffff_u64);
        observe_remote(&remote);
        let local = now();
        let local_logical = u64::from_str_radix(local.split('-').nth(1).unwrap(), 16).unwrap();
        assert!(local_logical > 0xffff, "local logical should be > remote logical");
    }

    #[test]
    fn observe_remote_ignores_malformed() {
        observe_remote("not-a-valid-hlc");
        observe_remote("");
    }
}
