mod classifier;
mod executor;
mod report;
mod scanner;
mod session;

pub use classifier::{classify_session, Classification, Confidence, Decision, Provider, Reason};
pub use executor::{trash_candidates, TrashReport};
pub use report::{explain_report, json_report, text_report};
pub use scanner::{explain_file, scan_roots, RootConfig, ScanConfig, ScanItem, ScanReport};
pub use session::{ParsedLine, SessionSample};
