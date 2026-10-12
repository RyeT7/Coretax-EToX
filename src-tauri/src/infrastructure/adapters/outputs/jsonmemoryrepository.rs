use std::{
    collections::HashMap,
    fs::File,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    application::ports::outbound::memoryrepository::MemoryRepository,
    domain::entities::{memory::Memory, tagmapping::TagMapping},
};

const MEMORY_FILE_VERSION: u32 = 1;
const MEMORY_FILE_NAME: &str = "memory.json";

fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".exceltoxml-write-test-{}", std::process::id()));

    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct MemoryFile {
    version: u32,
    invoice_number_column: String,
    good_service_identifier_column: String,
    tag_mappings: HashMap<String, MemoryFileMapping>,
    last_excel_dir: Option<String>,
    last_xml_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct MemoryFileMapping {
    mapped_column: Option<String>,
    default_value: Option<String>,
}

pub struct JsonMemoryRepository;

impl Default for JsonMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonMemoryRepository {
    pub fn new() -> Self {
        Self
    }
}

impl MemoryRepository for JsonMemoryRepository {
    fn load(&self, path: &str) -> Result<Option<Memory>, String> {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Failed to read memory file '{path}': {e}")),
        };

        let file: MemoryFile = serde_json::from_str(&contents)
            .map_err(|e| format!("Memory file '{path}' is not valid: {e}"))?;

        if file.version != MEMORY_FILE_VERSION {
            return Err(format!(
                "Memory file '{path}' has version {}, expected {MEMORY_FILE_VERSION}",
                file.version
            ));
        }

        Ok(Some(Memory {
            invoice_number_column: file.invoice_number_column,
            good_service_identifier_column: file.good_service_identifier_column,
            tag_mappings: file
                .tag_mappings
                .into_iter()
                .map(|(k, m)| (k, TagMapping::new(m.mapped_column, m.default_value)))
                .collect(),
            last_excel_dir: file.last_excel_dir,
            last_xml_dir: file.last_xml_dir,
        }))
    }

    fn save(&self, path: &str, memory: &Memory) -> Result<(), String> {
        let file = MemoryFile {
            version: MEMORY_FILE_VERSION,
            invoice_number_column: memory.invoice_number_column.clone(),
            good_service_identifier_column: memory.good_service_identifier_column.clone(),
            tag_mappings: memory
                .tag_mappings
                .iter()
                .map(|(k, m)| {
                    (
                        k.clone(),
                        MemoryFileMapping {
                            mapped_column: m.mapped_column.clone(),
                            default_value: m.default_value.clone(),
                        },
                    )
                })
                .collect(),
            last_excel_dir: memory.last_excel_dir.clone(),
            last_xml_dir: memory.last_xml_dir.clone(),
        };

        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize memory: {e}"))?;

        let tmp_path = format!("{path}.tmp");
        let result = File::create(&tmp_path)
            .and_then(|mut tmp| {
                tmp.write_all(json.as_bytes())?;
                tmp.sync_all()
            })
            .and_then(|_| std::fs::rename(&tmp_path, path));

        result.map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            format!("Failed to write memory to '{path}': {e}")
        })
    }

    fn default_location(
        &self,
        install_dir: Option<&Path>,
        app_data_dir: &Path,
    ) -> Result<String, String> {
        if let Some(dir) = install_dir.filter(|dir| is_writable(dir)) {
            return Ok(path_to_string(dir.join(MEMORY_FILE_NAME)));
        }

        std::fs::create_dir_all(app_data_dir).map_err(|e| {
            format!(
                "Failed to create app data folder '{}': {e}",
                app_data_dir.display()
            )
        })?;

        Ok(path_to_string(app_data_dir.join(MEMORY_FILE_NAME)))
    }
}
