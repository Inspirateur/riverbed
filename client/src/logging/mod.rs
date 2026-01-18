#[cfg(feature = "logging")]
pub mod log_display;


// Re-export LogInspectorPlugin from shared
#[cfg(feature = "logging")]
pub use shared::logging::LogInspectorPlugin;
