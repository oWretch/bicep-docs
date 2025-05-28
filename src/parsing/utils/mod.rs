//! Utility modules for Bicep parsing
//!
//! This module contains specialized utility functions organized by domain:
//! - decorators: Decorator parsing and extraction utilities
//! - types: Type parsing utilities for various Bicep type expressions
//! - values: Value parsing utilities for literals and expressions
//! - text: Text processing utilities for node extraction and string handling

pub mod decorators;
pub mod types;
pub mod values;
pub mod text;

// Re-export commonly used utilities
pub use decorators::{extract_description_from_decorators, parse_decorators, parse_decorator};
pub use types::{parse_property_type, parse_union_type, parse_array_type, parse_type_node};
pub use values::{parse_value_node, parse_array_items};
pub use text::{get_node_text, get_primitive_value};
