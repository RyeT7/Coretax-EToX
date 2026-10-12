use std::path::Path;

use crate::application::dto::request::memorydto::MemoryDTO;

pub trait MemoryUseCase: Send + Sync {
    fn load_memory(&self, path: &str) -> Result<Option<MemoryDTO>, String>;
    fn save_memory(&self, path: &str, memory: &MemoryDTO) -> Result<(), String>;
    fn default_memory_path(
        &self,
        install_dir: Option<&Path>,
        app_data_dir: &Path,
    ) -> Result<String, String>;
}
