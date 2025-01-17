use inquire::ui::{Color, RenderConfig, StyleSheet};
use inquire::{Confirm, InquireError, Select, Text};

/// Text input prompt.
pub fn text_input(message: &str) -> Result<String, InquireError> {
    Text::new(message).prompt()
}

/// Text input prompt with default value.
pub fn text_input_with_default(message: &str, default: &str) -> Result<String, InquireError> {
    Text::new(message).with_default(default).prompt()
}

/// Confirm prompt.
pub fn confirm(message: &str, default: bool) -> Result<bool, InquireError> {
    Confirm::new(message).with_default(default).prompt()
}

/// Confirm prompt for database initializing.
pub fn confirm_init() -> Result<bool, InquireError> {
    Confirm::new("Initializing existing database?")
        .with_default(false)
        .with_help_message("Warning: All existing data will be deleted")
        .with_render_config(help_warning())
        .prompt()
}

/// Confirm prompt for task name input.
pub fn confirm_taskname_input(
    level: u8,
    current: &Option<String>,
    default: bool,
) -> Result<bool, InquireError> {
    let current_value = match current {
        Some(s) => s,
        None => "",
    };
    Confirm::new(&format!("Set level {} ({})?", level, current_value))
        .with_default(default)
        .prompt()
}

/// Select prompt.
pub fn select(candidates: Vec<String>, message: &str) -> Result<String, InquireError> {
    Select::new(message, candidates).prompt()
}

/// Warning color config.
fn help_warning<'a>() -> RenderConfig<'a> {
    RenderConfig::default().with_help_message(StyleSheet::default().with_fg(Color::LightRed))
}
