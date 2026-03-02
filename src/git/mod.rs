pub mod churn;
pub mod history;

pub use churn::{analyze_churn, ChurnMetrics, ChurnTrend};
pub use history::{analyze_git, GitMetrics};
