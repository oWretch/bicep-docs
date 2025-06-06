//! Utility modules for Bicep parsing
//!
//! This module contains specialized utility functions organized by domain:
//! - decorators: Decorator parsing and extraction utilities
//! - types: Type parsing utilities for various Bicep type expressions
//! - values: Value parsing utilities for literals and expressions
//! - text: Text processing utilities for node extraction and string handling

pub mod decorators;
pub mod text;
pub mod types;
pub mod values;

// Re-export commonly used utilities
pub use decorators::{extract_description_from_decorators, parse_decorator, parse_decorators};
pub use text::{get_node_text, get_primitive_value};
pub use types::{parse_array_type, parse_property_type, parse_type_node, parse_union_type};
pub use values::{parse_array_items, parse_value_node};
