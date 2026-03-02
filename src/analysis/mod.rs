pub mod collector;
pub mod complexity;
pub mod documentation;
pub mod structure;
pub mod tests;

pub use collector::{AnalysisResult, Collector};
pub use complexity::{analyze_complexity, ComplexityMetrics};
pub use documentation::{analyze_documentation, check_readme, DocumentationMetrics};
pub use structure::{analyze_structure, StructureMetrics};
pub use tests::{analyze_tests, TestMetrics};
