pub mod logging;
mod log_replay;
mod log_inspector;

// Re-export LogReplayPlugin for log inspector mode (reads from log file)
#[cfg(feature = "logging")]
pub use log_replay::LogReplayPlugin;

// Re-export LogInspectorPlugin (processes LogEvent messages into inspector state)
#[cfg(feature = "logging")]
pub use log_inspector::LogInspectorPlugin;