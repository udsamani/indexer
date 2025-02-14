use config::Config;

use crate::AppResult;

/// Context for the application
///
/// The context is a container for the service configuration.
#[derive(Clone)]
pub struct Context {
    /// Context Name
    pub name: String,

    /// Configuration
    pub config: Config,

    /// Broadcaster
    pub app: AppBroadcaster,
}


#[derive(Debug, Clone, Default)]
pub enum AppMessage {
    /// Clean Exit
    #[default]
    Exit,
    /// Exit on Failure
    ExitOnFailure,
}

pub type AppBroadcaster = tokio::sync::broadcast::Sender<AppMessage>;


impl Context {

    pub fn from_config(config: Config) -> Self {
        let name = config.get_string("app_name").unwrap_or("default".to_string());
        let broadcaster = tokio::sync::broadcast::Sender::new(10);
        Self { name, config, app: broadcaster }
    }

    pub fn with_name(&self, name: &str) -> Self {
        Self { name: name.to_string(), config: self.config.clone(), app: self.app.clone() }
    }

    pub fn with_config(&self, config: Config) -> Self {
        Self { name: self.name.clone(), config, app: self.app.clone() }
    }

    pub fn exit(&self) -> bool {
        self.app.send(AppMessage::Exit).is_ok()
    }

    pub fn exit_on_failure(&self) -> bool {
        self.app.send(AppMessage::ExitOnFailure).is_ok()
    }

    pub fn log_and_exit(&self, message: &str) -> AppResult<String> {
        log::info!("exiting app = {} message = {}", self.name, message);
        Ok(message.to_string())
    }
}
