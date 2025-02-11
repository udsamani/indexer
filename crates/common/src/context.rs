use crate::{AppConfigRef, AppResult};

/// Context for the application
///
/// The context is a container for the service configuration.
pub struct Context {
    /// Context Name
    pub name: String,

    /// Configuration
    pub config: AppConfigRef,
}


impl Context {

    pub fn from_config(config: AppConfigRef) -> Self {
        let name = config.get_app_name();
        Self { name, config }
    }

    pub fn exit(&self) -> bool {
        false
    }

    pub fn log_and_exit(&self, message: &str) -> AppResult<String> {
        log::info!("exiting app = {} message = {}", self.name, message);
        Ok(message.to_string())
    }
}
