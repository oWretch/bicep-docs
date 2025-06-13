//! Core types for the Bicep document model
//!
//! This module re-exports all the core types from the parsing module
//! to provide a clean public API for the library.

// Re-export the types from parsing module
pub use crate::parsing::{
    BicepCustomType, BicepDecorator, BicepDocument, BicepFunction, 
    BicepFunctionArgument, BicepImport, BicepImportSymbol, BicepModule, 
    BicepOutput, BicepParameter, BicepResource, BicepType, BicepValue, 
    BicepVariable, ModuleSource,
};