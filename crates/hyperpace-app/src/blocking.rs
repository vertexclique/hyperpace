//! Runs a device or store call (both synchronous, and a device call can wait up to
//! [`crate::state::REQUEST_TIMEOUT`] for a reply) on Tauri's blocking thread pool instead of the
//! async executor, so one slow device does not stall every other in-flight command.

use crate::error::AppError;

/// Run `f` on the blocking pool and return its result.
///
/// # Errors
///
/// Returns whatever `f` itself returns, and [`AppError::Tauri`] wrapping the background task's
/// own failure if it could not be joined (only on an unexpected runtime shutdown).
pub async fn blocking<T, F>(f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f).await?
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn a_successful_closure_returns_its_value() {
        let result = tauri::async_runtime::block_on(blocking(|| Ok::<_, AppError>(42)));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn a_failing_closure_returns_its_error() {
        let result =
            tauri::async_runtime::block_on(blocking(|| Err::<u8, _>(AppError::NotConnected)));
        assert!(matches!(result, Err(AppError::NotConnected)));
    }
}
