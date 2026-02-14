use std::sync::Mutex;

use crate::models::{AppConfig, Context};

pub struct AppState {
    pub contexts: Mutex<Vec<Context>>,
    pub config: Mutex<AppConfig>,
}
