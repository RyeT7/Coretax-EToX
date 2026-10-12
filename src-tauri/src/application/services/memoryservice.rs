use std::{path::Path, sync::Arc};

use crate::application::{
    dto::request::memorydto::MemoryDTO,
    ports::{inbound::memoryusecase::MemoryUseCase, outbound::memoryrepository::MemoryRepository},
};

pub struct MemoryService {
    memory_repository: Arc<dyn MemoryRepository>,
}

impl MemoryService {
    pub fn new(memory_repository: Arc<dyn MemoryRepository>) -> Self {
        MemoryService { memory_repository }
    }
}

impl MemoryUseCase for MemoryService {
    fn load_memory(&self, path: &str) -> Result<Option<MemoryDTO>, String> {
        Ok(self.memory_repository.load(path)?.map(MemoryDTO::from))
    }

    fn save_memory(&self, path: &str, memory: &MemoryDTO) -> Result<(), String> {
        self.memory_repository.save(path, &memory.into())
    }

    fn default_memory_path(
        &self,
        install_dir: Option<&Path>,
        app_data_dir: &Path,
    ) -> Result<String, String> {
        self.memory_repository
            .default_location(install_dir, app_data_dir)
    }
}
