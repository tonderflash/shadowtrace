mod dashboard;
mod process_monitor;
mod file_monitor;
mod network_monitor;
mod reports;
mod help;

pub use dashboard::draw_dashboard;
pub use process_monitor::draw_process_monitor;
pub use file_monitor::draw_file_monitor;
pub use network_monitor::draw_network_monitor;
pub use reports::draw_reports;
pub use help::draw_help; 
