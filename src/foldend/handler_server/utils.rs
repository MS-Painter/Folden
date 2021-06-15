use crate::mapping::Mapping;

pub fn is_concurrent_handlers_limit_reached(
    mapping: &Mapping,
    concurrent_threads_limit: u8,
) -> bool {
    let mut live_handlers_count: u8 = 0;
    if live_handlers_count >= concurrent_threads_limit {
        return true;
    }
    for _ in mapping.iter_live_handlers() {
        live_handlers_count += 1;
        if live_handlers_count >= concurrent_threads_limit {
            return true;
        }
    }
    false
}
