use rustikv_telemetry_gateway::query::bucket_ranges;

#[test]
fn splits_window_into_step_sized_buckets() {
    // [100, 280) with step 60 -> buckets [100,160),[160,220),[220,280)
    let b = bucket_ranges(100, 280, 60);
    assert_eq!(b, vec![(100, 160), (160, 220), (220, 280)]);
}

#[test]
fn last_bucket_clamps_to_end() {
    let b = bucket_ranges(100, 250, 60);
    assert_eq!(b, vec![(100, 160), (160, 220), (220, 250)]);
}

#[test]
fn empty_when_from_ge_to() {
    assert!(bucket_ranges(200, 100, 60).is_empty());
}
