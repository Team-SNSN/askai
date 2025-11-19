pub mod protocol;
pub mod session;
pub mod server;

// Note: DaemonRequest, DaemonResponse, and DaemonServer are accessed via full path
// (e.g., daemon::protocol::DaemonRequest) rather than re-exports
