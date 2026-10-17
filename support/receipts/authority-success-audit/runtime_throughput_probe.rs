#![allow(dead_code,private_interfaces)]
extern crate alloc;
mod roof {
use core::time::Duration;

use alloc::vec::Vec;

/// A resource's roofline record: how much of it a run must move, against the
/// peak rate that resource can sustain.
///
/// Neutral by design: nothing here says whether `amount` counts bytes or
/// operations, or where `peak_per_s` came from, measured or modeled. That is
/// what lets [`AutotuneBound`](crate::tune::AutotuneBound) compose this with
/// a threshold instead of reimplementing the arithmetic.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(serializable, derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceBound {
    /// How much of the resource the run must move.
    pub amount: usize,
    /// The peak rate this resource can sustain, in the same unit as `amount`
    /// per second.
    pub peak_per_s: f64,
}

impl ResourceBound {
    /// Time `amount` would take running at `peak_per_s`, with no allowance
    /// for anything else the kernel does concurrently.
    ///
    /// `None` for a `peak_per_s` that is zero, negative, `NaN`, or infinite:
    /// none of those describe a real ceiling to divide by.
    pub fn time_at_peak(&self) -> Option<Duration> {
        if self.peak_per_s.is_normal() {
            Some(Duration::from_secs_f64(
                self.amount as f64 / self.peak_per_s,
            ))
        } else {
            None
        }
    }
}

/// The roofline convention this module uses throughout: a run cannot finish
/// faster than its slowest resource allows even under perfect overlap
/// between them, so that resource is the one binding the achievable
/// duration, whatever the others manage. [`binding_achieved`] applies the
/// same reduction to a run's actually measured rates.
///
/// The resource requiring the most time even at its own peak: the entry with
/// the largest [`ResourceBound::time_at_peak`].
///
/// `None` if every entry's `time_at_peak` is `None`, or `bounds` is empty.
pub fn binding_resource(bounds: &[ResourceBound]) -> Option<&ResourceBound> {
    bounds
        .iter()
        .filter(|bound| bound.time_at_peak().is_some())
        .max_by_key(|bound| bound.time_at_peak())
}

/// One resource's achieved rate during a measured run, against its modeled
/// peak.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AchievedThroughput {
    /// Achieved rate: `amount / duration`.
    pub achieved_per_s: f64,
    /// `achieved_per_s / peak_per_s`, as a fraction. Not clamped to `[0, 1]`:
    /// a run beating the modeled peak is a finding about the model, not an
    /// error to hide.
    pub fraction_of_peak: f64,
}

/// Scores an actual run against a set of [`ResourceBound`]s, one resource at
/// a time.
///
/// `duration` is the run's actual, measured duration, shared by every bound:
/// they all describe the same execution, so the same wall time buys each
/// resource a different achieved rate against its own peak. Results come
/// back in `bounds` order. A zero duration yields `NaN` achieved rates rather
/// than dividing by zero.
pub fn score_resources(duration: Duration, bounds: &[ResourceBound]) -> Vec<AchievedThroughput> {
    bounds
        .iter()
        .map(|bound| {
            let achieved_per_s = if duration.is_zero() {
                f64::NAN
            } else {
                bound.amount as f64 / duration.as_secs_f64()
            };

            AchievedThroughput {
                achieved_per_s,
                fraction_of_peak: achieved_per_s / bound.peak_per_s,
            }
        })
        .collect()
}

/// The resource that actually governed the run's duration: [`binding_resource`]'s
/// reduction applied to achieved scores instead of raw bounds. Every entry
/// here shares one `duration`, so ranking by `fraction_of_peak` gives the
/// same order ranking the underlying bounds by `time_at_peak` would.
///
/// `NaN` entries (a zero duration, or a zero, negative, or non-finite peak)
/// cannot be compared and are skipped; `None` if every entry is such, or
/// `scores` is empty.
pub fn binding_achieved(scores: &[AchievedThroughput]) -> Option<&AchievedThroughput> {
    scores
        .iter()
        .filter(|score| score.fraction_of_peak.is_finite())
        .max_by(|a, b| a.fraction_of_peak.total_cmp(&b.fraction_of_peak))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bound(amount: usize, peak_per_s: f64) -> ResourceBound {
        ResourceBound { amount, peak_per_s }
    }

    #[test]
    fn time_at_peak_is_amount_over_peak() {
        assert_eq!(bound(8, 4.0).time_at_peak(), Some(Duration::from_secs(2)));
    }

    #[test]
    fn time_at_peak_is_none_for_a_non_normal_peak() {
        assert_eq!(bound(8, 0.0).time_at_peak(), None);
        assert_eq!(bound(8, f64::NAN).time_at_peak(), None);
        assert_eq!(bound(8, f64::INFINITY).time_at_peak(), None);
    }

    #[test]
    fn binding_resource_is_the_one_needing_the_most_time_at_peak() {
        // 8 ops at 4 ops/s takes 2s at peak; 8 ops at 8 ops/s takes 1s: the
        // first resource is the one that would still take longer even
        // running flat out, so it is the one that binds.
        let slower = bound(8, 4.0);
        let faster = bound(8, 8.0);

        assert_eq!(binding_resource(&[slower, faster]), Some(&slower));
    }

    #[test]
    fn binding_resource_skips_non_normal_peaks_and_is_none_if_all_are() {
        let unusable = bound(8, 0.0);
        let usable = bound(8, 4.0);

        assert_eq!(binding_resource(&[unusable, usable]), Some(&usable));
        assert_eq!(binding_resource(&[unusable]), None);
        assert_eq!(binding_resource(&[]), None);
    }

    #[test]
    fn score_resources_reports_achieved_rate_and_fraction_of_peak() {
        let bounds = [bound(100, 200.0), bound(400, 800.0)];

        let scores = score_resources(Duration::from_secs(1), &bounds);

        assert_eq!(scores[0].achieved_per_s, 100.0);
        assert_eq!(scores[0].fraction_of_peak, 0.5);
        assert_eq!(scores[1].achieved_per_s, 400.0);
        assert_eq!(scores[1].fraction_of_peak, 0.5);
    }

    #[test]
    fn a_zero_duration_reports_nan_instead_of_dividing_by_zero() {
        let scores = score_resources(Duration::ZERO, &[bound(100, 200.0)]);

        assert!(scores[0].achieved_per_s.is_nan());
        assert!(scores[0].fraction_of_peak.is_nan());
    }

    /// A matmul-shaped run: large A/B reads against a read peak, a small C
    /// write against a write peak. The two must score independently, and the
    /// read, which alone would still need more time even at its own peak
    /// (0.9s vs the write's 0.5s), must be the one that binds even though it
    /// moves far more bytes than the write, not fewer.
    #[test]
    fn resources_with_different_peaks_score_independently_and_pick_the_slower_one() {
        let duration = Duration::from_secs(1);
        let read = bound(900_000, 1_000_000.0); // 0.9s at peak
        let write = bound(100_000, 200_000.0); // 0.5s at peak

        assert_eq!(binding_resource(&[read, write]), Some(&read));

        let scores = score_resources(duration, &[read, write]);

        assert_eq!(scores[0].achieved_per_s, 900_000.0);
        assert_eq!(scores[0].fraction_of_peak, 0.9);
        assert_eq!(scores[1].achieved_per_s, 100_000.0);
        assert_eq!(scores[1].fraction_of_peak, 0.5);

        let binding = binding_achieved(&scores).unwrap();
        assert_eq!(binding.fraction_of_peak, 0.9);
    }

    #[test]
    fn binding_achieved_skips_non_finite_entries_and_is_none_if_all_are() {
        let finite = AchievedThroughput {
            achieved_per_s: 10.0,
            fraction_of_peak: 0.4,
        };
        let non_finite = AchievedThroughput {
            achieved_per_s: f64::NAN,
            fraction_of_peak: f64::NAN,
        };

        assert_eq!(
            binding_achieved(&[non_finite, finite])
                .unwrap()
                .fraction_of_peak,
            0.4
        );
        assert!(binding_achieved(&[non_finite]).is_none());
        assert!(binding_achieved(&[]).is_none());
    }
}

#[cfg(test)]mod audit {
use super::*;
#[test]fn a_negative_normal_peak_panics_instead_of_being_ignored(){
 let negative=ResourceBound{amount:8,peak_per_s:-4.0};assert!((-4.0f64).is_normal());
 assert!(std::panic::catch_unwind(||negative.time_at_peak()).is_err());
}
#[test]fn a_positive_finite_peak_can_overflow_duration(){
 let slow=ResourceBound{amount:8,peak_per_s:1e-100};assert!(slow.peak_per_s.is_normal());
 assert!(std::panic::catch_unwind(||slow.time_at_peak()).is_err());
}
#[test]fn scoring_a_negative_peak_keeps_a_binding_the_docs_say_is_skipped(){
 let scores=score_resources(Duration::from_secs(1),&[ResourceBound{amount:8,peak_per_s:-4.0}]);
 let chosen=binding_achieved(&scores).unwrap();assert_eq!(chosen.fraction_of_peak,-2.0);
}
#[test]fn scoring_an_infinite_peak_keeps_a_binding_the_docs_say_is_skipped(){
 let scores=score_resources(Duration::from_secs(1),&[ResourceBound{amount:8,peak_per_s:f64::INFINITY}]);
 assert_eq!(binding_achieved(&scores).unwrap().fraction_of_peak,0.0);
}
#[test]fn scoring_a_zero_peak_control_is_unusable(){
 let scores=score_resources(Duration::from_secs(1),&[ResourceBound{amount:8,peak_per_s:0.0}]);
 assert!(binding_achieved(&scores).is_none());
}
}
}
mod ranking {
use std::time::Duration;
const TARGET_DURATION: Duration = Duration::from_millis(20);
const RANK_SAMPLES:usize=3;
struct KernelConfig{sample:Box<dyn Fn(usize)->Duration>,ops_count:usize,min_iterations:usize}
struct ThroughputValue{ops_count:usize,duration:Duration}
struct Ranked{value:ThroughputValue,iterations:usize}
struct ThroughputBenchmarker;
impl ThroughputBenchmarker{
pub fn rank(kernel_config: &KernelConfig, iterations: usize) -> Ranked {
        let settling = iterations.max(kernel_config.min_iterations).max(1);
        // A first launch is what compiling and faulting in cost, so it is spent
        // and the count settled from a second.
        let _ = (kernel_config.sample)(settling);
        let took = (kernel_config.sample)(settling);

        let iterations = Self::retarget(settling, took)
            .max(kernel_config.min_iterations)
            .max(1);

        let mut fastest = Duration::MAX;
        for _ in 0..RANK_SAMPLES {
            fastest = fastest.min((kernel_config.sample)(iterations));
        }

        let duration = fastest / iterations as u32;

        Ranked {
            value: ThroughputValue {
                ops_count: kernel_config.ops_count,
                duration,
            },
            iterations,
        }
    }
fn retarget(iterations: usize, took: Duration) -> usize {
        let took = took.as_secs_f64();

        if took <= 0.0 {
            return iterations;
        }

        let scaled = iterations as f64 * (TARGET_DURATION.as_secs_f64() / took);

        if scaled.is_finite() {
            (scaled as usize).max(1)
        } else {
            iterations
        }
    }
}
#[cfg(test)]mod audit{
use super::*;
#[test]fn ordinary_ranking_control_preserves_duration_per_iteration(){
 let config=KernelConfig{sample:Box::new(|n|Duration::from_nanos(1000*n as u64)),ops_count:1,min_iterations:1};
 let ranked=ThroughputBenchmarker::rank(&config,1000);assert_eq!(ranked.iterations,20_000);assert_eq!(ranked.value.duration,Duration::from_nanos(1000));
}
#[test]fn zero_minimum_control_is_clamped_to_one(){
 let config=KernelConfig{sample:Box::new(|_|TARGET_DURATION),ops_count:1,min_iterations:0};
 let ranked=ThroughputBenchmarker::rank(&config,0);assert_eq!(ranked.iterations,1);assert_eq!(ranked.value.duration,TARGET_DURATION);
}
#[cfg(target_pointer_width="64")]
#[test]fn a_usize_iteration_floor_multiple_of_u32_range_divides_by_zero(){
 let count=1usize<<32;
 let config=KernelConfig{sample:Box::new(|_|TARGET_DURATION),ops_count:1,min_iterations:count};
 assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||ThroughputBenchmarker::rank(&config,count))).is_err());
}
#[cfg(target_pointer_width="64")]
#[test]fn retargeting_can_leave_warmup_ceiling_and_normalize_by_wrapped_count(){
 let config=KernelConfig{sample:Box::new(|n|if n==1000 {Duration::from_nanos(1)}else{Duration::from_secs(20)}),ops_count:1,min_iterations:1};
 let ranked=ThroughputBenchmarker::rank(&config,1000);
 assert_eq!(ranked.iterations,20_000_000_000);
 assert_eq!(ranked.value.duration,Duration::from_nanos(7));
 let correct=Duration::from_secs_f64(20.0/ranked.iterations as f64);assert_eq!(correct,Duration::from_nanos(1));assert_ne!(ranked.value.duration,correct);
}
}
}
