use std::path::Path;

use crate::domain::entities::memory::Memory;

pub trait MemoryRepository: Send + Sync {
    fn load(&self, path: &str) -> Result<Option<Memory>, String>;

    fn save(&self, path: &str, memory: &Memory) -> Result<(), String>;

    fn default_location(
        &self,
        install_dir: Option<&Path>,
        app_data_dir: &Path,
    ) -> Result<String, String>;
}
