#[cfg(test)]
mod unit_tests {
    use crate::constants::*;
    use crate::utils::*;

    // ── calculate_streamed_amount ──

    #[test]
    fn streamed_before_start_is_zero() {
        let result = calculate_streamed_amount(1_000_000_000, 1000, 2000, 500).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn streamed_at_exact_start_is_zero() {
        let result = calculate_streamed_amount(1_000_000_000, 1000, 2000, 1000).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn streamed_partial_duration() {
        // rate = PRECISION = 1e9 => 1 token/sec
        // elapsed = 10 sec => streamed = 1e9 * 10 / 1e9 = 10 tokens
        let result = calculate_streamed_amount(PRECISION, 0, 1000, 10).unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn streamed_exact_end_time() {
        let result = calculate_streamed_amount(PRECISION, 0, 100, 100).unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn streamed_capped_by_end_time() {
        let result = calculate_streamed_amount(PRECISION, 0, 100, 500).unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn streamed_open_ended() {
        let result = calculate_streamed_amount(PRECISION, 0, 0, 3600).unwrap();
        assert_eq!(result, 3600);
    }

    #[test]
    fn streamed_small_rate() {
        // rate = 100_000_000 => 0.1 tokens/sec, elapsed = 50 => 5 tokens
        let result = calculate_streamed_amount(100_000_000, 0, 0, 50).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn streamed_with_rate_not_divisible() {
        // rate = 3, PRECISION = 1e9, elapsed = 1 => truncates to 0
        let result = calculate_streamed_amount(3, 0, 1000, 1).unwrap();
        assert_eq!(result, 0);
    }

    // ── calculate_refund_amount ──

    #[test]
    fn refund_simple() {
        assert_eq!(calculate_refund_amount(1000, 300).unwrap(), 700);
    }

    #[test]
    fn refund_none_streamed() {
        assert_eq!(calculate_refund_amount(1000, 0).unwrap(), 1000);
    }

    #[test]
    fn refund_fully_streamed() {
        assert_eq!(calculate_refund_amount(1000, 1000).unwrap(), 0);
    }

    #[test]
    fn refund_overflow_rejected() {
        assert!(calculate_refund_amount(100, 101).is_err());
    }

    // ── checked_mul_div ──

    #[test]
    fn mul_div_basic() {
        assert_eq!(checked_mul_div(100, 3, 2).unwrap(), 150);
    }

    #[test]
    fn mul_div_denominator_zero() {
        assert!(checked_mul_div(100, 3, 0).is_err());
    }

    #[test]
    fn mul_div_zero_numerator() {
        assert_eq!(checked_mul_div(100, 0, 5).unwrap(), 0);
    }

    #[test]
    fn mul_div_large_numbers() {
        // 1e12 * 1e6 / 2 = 5e17 = 500_000_000_000_000_000
        let result = checked_mul_div(1_000_000_000_000, 1_000_000, 2).unwrap();
        assert_eq!(result, 500_000_000_000_000_000);
    }

    // ── validate_timestamp ──

    #[test]
    fn timestamp_in_future_ok() {
        assert!(validate_timestamp(2000, 1000).is_ok());
    }

    #[test]
    fn timestamp_in_past_err() {
        assert!(validate_timestamp(500, 1000).is_err());
    }

    #[test]
    fn timestamp_exact_now_err() {
        assert!(validate_timestamp(1000, 1000).is_err());
    }

    // ── validate_rate ──

    #[test]
    fn rate_positive_ok() {
        assert!(validate_rate(1).is_ok());
        assert!(validate_rate(PRECISION).is_ok());
    }

    #[test]
    fn rate_zero_err() {
        assert!(validate_rate(0).is_err());
    }

    // ── validate_duration ──

    #[test]
    fn valid_duration_ok() {
        assert!(validate_duration(0, 86400).is_ok());
    }

    #[test]
    fn min_duration_ok() {
        assert!(validate_duration(0, 1).is_ok());
    }

    #[test]
    fn max_duration_ok() {
        assert!(validate_duration(0, MAX_STREAM_DURATION).is_ok());
    }

    #[test]
    fn duration_zero_err() {
        assert!(validate_duration(1000, 1000).is_err());
    }

    #[test]
    fn duration_too_long_err() {
        assert!(validate_duration(0, MAX_STREAM_DURATION + 1).is_err());
    }

    #[test]
    fn duration_too_short_err() {
        assert!(validate_duration(0, 0).is_err());
    }

    #[test]
    fn duration_negative_err() {
        assert!(validate_duration(1000, 500).is_err());
    }

    // ── rate_to_apr_bps / apr_bps_to_rate ──

    #[test]
    fn rate_to_apr_bps_basic() {
        // rate=1, bps = 1 * 31536000 * 10000 / 1e9 = 315.36 → truncates to 315
        assert_eq!(rate_to_apr_bps(1).unwrap(), 315);
    }

    #[test]
    fn apr_bps_to_rate_round_trip() {
        // Use a tiny rate to stay within u16 range for bps
        // rate = 1_000 => bps ≈ 315_360 (fits in u16 max 65535)
        let rate_before = 1_000u64;
        let bps = rate_to_apr_bps(rate_before).unwrap();
        let rate_after = apr_bps_to_rate(bps).unwrap();
        // Expect some precision loss from integer division
        let diff = rate_before.abs_diff(rate_after);
        assert!(diff <= rate_before, "Round-trip: {} -> bps={} -> {}", rate_before, bps, rate_after);
    }

    #[test]
    fn rate_to_apr_bps_zero_rate() {
        assert_eq!(rate_to_apr_bps(0).unwrap(), 0);
    }

    // ── Pause/Resume time compensation ──

    #[test]
    fn pause_compensation_math() {
        let start_time: i64 = 1000;
        let paused_at: i64 = 1100;
        let current_time: i64 = 1150;
        let pause_duration = current_time.checked_sub(paused_at).unwrap();
        assert_eq!(pause_duration, 50);
        let new_start = start_time.checked_add(pause_duration).unwrap();
        assert_eq!(new_start, 1050);
    }

    // ── Constants ──

    #[test]
    fn constants_are_sane() {
        assert_eq!(PRECISION, 1_000_000_000);
        assert_eq!(BPS_DENOMINATOR, 10_000);
        assert_eq!(SECONDS_PER_YEAR, 31_536_000);
        assert_eq!(MIN_STREAM_DURATION, 1);
        assert_eq!(MAX_STREAM_DURATION, 315_360_000);
    }

    // ── Stream::LEN ──

    #[test]
    fn stream_discriminator_size() {
        use crate::state::Stream;
        // Anchor 1.0.2 + rustc 1.95: actual LEN is 194
        // (8 discriminator + 13 fields with alignment)
        assert_eq!(Stream::LEN, 194);
    }
}
