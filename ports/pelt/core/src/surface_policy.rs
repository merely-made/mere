//! Host-owned limits for polling expensive live surface producers.
//!
//! A surface can remain activated and keep its last composed image while this
//! policy defers another producer poll.  That keeps resource pressure separate
//! from tile visibility and keyboard focus: those remain routing concerns.

/// Refresh limits for live composited surfaces in one workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceResourcePolicy {
    /// Maximum number of surface producers polled by one frame pass.
    pub max_refreshes_per_frame: usize,
    /// Poll each admitted producer once every N frame passes.
    pub refresh_every_n_frames: u32,
}

impl Default for SurfaceResourcePolicy {
    fn default() -> Self {
        Self {
            max_refreshes_per_frame: 4,
            refresh_every_n_frames: 1,
        }
    }
}

impl SurfaceResourcePolicy {
    /// Clamp malformed settings while retaining useful live work.
    pub const fn normalized(self) -> Self {
        Self {
            max_refreshes_per_frame: self.max_refreshes_per_frame,
            refresh_every_n_frames: if self.refresh_every_n_frames == 0 {
                1
            } else {
                self.refresh_every_n_frames
            },
        }
    }

    /// Whether a producer is eligible during this frame pass.
    pub const fn admits(self, frame_index: u64, refreshes: usize) -> bool {
        let policy = self.normalized();
        refreshes < policy.max_refreshes_per_frame
            && frame_index % (policy.refresh_every_n_frames as u64) == 0
    }
}

/// Select a rotating window so a small per-frame cap cannot starve later
/// activated surfaces. The caller retains presentation order separately.
pub fn rotated_indices(len: usize, cursor: usize, limit: usize) -> Vec<usize> {
    if len == 0 {
        return Vec::new();
    }
    let start = cursor % len;
    (0..limit.min(len)).map(|offset| (start + offset) % len).collect()
}

#[cfg(test)]
mod tests {
    use super::SurfaceResourcePolicy;

    #[test]
    fn cadence_and_per_frame_cap_are_independent() {
        let policy = SurfaceResourcePolicy {
            max_refreshes_per_frame: 2,
            refresh_every_n_frames: 3,
        };
        assert!(policy.admits(0, 0));
        assert!(policy.admits(3, 1));
        assert!(!policy.admits(3, 2));
        assert!(!policy.admits(4, 0));
    }

    #[test]
    fn zero_cadence_is_safe_and_does_not_disable_refresh() {
        let policy = SurfaceResourcePolicy {
            max_refreshes_per_frame: 1,
            refresh_every_n_frames: 0,
        };
        assert!(policy.admits(17, 0));
    }

    #[test]
    fn rotating_window_eventually_visits_three_surfaces_with_cap_one() {
        let mut cursor = 0;
        let mut seen = [false; 3];
        for _ in 0..3 {
            let selected = super::rotated_indices(3, cursor, 1);
            seen[selected[0]] = true;
            cursor = (cursor + selected.len()) % 3;
        }
        assert_eq!(seen, [true, true, true]);
    }

    #[test]
    fn cadence_does_not_advance_rotation_on_skipped_frames() {
        let policy = SurfaceResourcePolicy {
            max_refreshes_per_frame: 1,
            refresh_every_n_frames: 3,
        };
        let mut cursor = 0;
        let mut seen = [false; 3];
        for frame in 0..9 {
            if policy.admits(frame, 0) {
                let selected = super::rotated_indices(3, cursor, 1);
                seen[selected[0]] = true;
                cursor = (cursor + 1) % 3;
            }
        }
        assert_eq!(seen, [true, true, true]);
        assert_eq!(cursor, 0);
    }
}
