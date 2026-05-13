pub fn safe_unwrap_or_default<T: Default>(result: Result<T, impl std::fmt::Display>, context: &str) -> T {
    result.unwrap_or_else(|e| {
        tracing::error!("{}: {}", context, e);
        T::default()
    })
}

pub fn safe_unwrap_or<T>(result: Result<T, impl std::fmt::Display>, context: &str, default: T) -> T {
    result.unwrap_or_else(|e| {
        tracing::error!("{}: {}", context, e);
        default
    })
}

pub fn safe_unwrap_none_or<T>(result: Result<T, impl std::fmt::Display>, context: &str, value: T) -> T {
    match result {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{}: {}", context, e);
            value
        }
    }
}
