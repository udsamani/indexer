

pub type AppConfigRef = Box<dyn AppConfig + Send + Sync>;

/// A trait representing the application configuration
pub trait AppConfig {

    /// Returns the name of the application
    fn get_app_name(&self) -> String;
}
