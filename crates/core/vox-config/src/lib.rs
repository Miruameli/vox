//! Layered configuration schema, loading, and strict validation.

mod files;
mod validation;

pub mod loader;
pub mod schema;

pub use loader::{discover_project_config, load, FlagOverrides, LoadInputs, PROJECT_CONFIG_NAME};
pub use schema::Config;
