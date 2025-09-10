//! Command implementations for the CLI

pub mod info;
pub mod list;
pub mod new;
pub mod update;

pub use info::run as show_template_info;
pub use list::run as list_templates;
pub use new::run as create_from_template;
