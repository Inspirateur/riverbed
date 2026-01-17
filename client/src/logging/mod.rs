pub mod log_display;

pub use log_display::InspectorDisplayPlugin;

// Re-export LogInspectorPlugin from shared
#[cfg(feature = "logging")]
pub use shared::logging::LogInspectorPlugin;
