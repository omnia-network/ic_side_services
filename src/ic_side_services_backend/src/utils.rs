/// Returns the current timestamp in nanoseconds.
pub fn get_current_timestamp_ns() -> u64 {
    ic_cdk::api::time()
}

/// Returns the current timestamp in milliseconds.
pub fn get_current_timestamp_ms() -> u64 {
    get_current_timestamp_ns() / 1_000_000
}
