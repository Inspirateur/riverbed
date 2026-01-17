mod log_inspector;
mod log_replay;
pub mod logging;

// Re-export LogReplayPlugin for log inspector mode (reads from log file)
#[cfg(feature = "logging")]
pub use log_replay::LogReplayPlugin;

// Re-export LogInspectorPlugin (processes LogEvent messages into inspector state)
#[cfg(feature = "logging")]
pub use log_inspector::LogInspectorPlugin;
