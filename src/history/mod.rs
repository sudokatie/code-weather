//! Historical metrics storage and trend analysis
//!
//! Stores weather reports over time to enable trend visualization
//! and regression warnings.

pub mod storage;
pub mod trend;

pub use storage::{HistoryEntry, HistoryStore};
pub use trend::{Trend, TrendAnalysis, TrendDirection};
