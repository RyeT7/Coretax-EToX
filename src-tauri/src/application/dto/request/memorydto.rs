use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    application::dto::request::tagmappingdto::TagMappingDTO,
    domain::entities::{memory::Memory, tagmapping::TagMapping},
};

#[derive(Serialize, Deserialize)]
pub struct MemoryDTO {
    pub invoice_number_column: String,
    pub good_service_identifier_column: String,
    pub tag_mappings: HashMap<String, TagMappingDTO>,
    pub last_excel_dir: Option<String>,
    pub last_xml_dir: Option<String>,
}

impl From<&MemoryDTO> for Memory {
    fn from(dto: &MemoryDTO) -> Self {
        Memory {
            invoice_number_column: dto.invoice_number_column.clone(),
            good_service_identifier_column: dto.good_service_identifier_column.clone(),
            tag_mappings: dto
                .tag_mappings
                .iter()
                .map(|(k, m)| {
                    (
                        k.clone(),
                        TagMapping::new(m.mapped_column.clone(), m.default_value.clone()),
                    )
                })
                .collect(),
            last_excel_dir: dto.last_excel_dir.clone(),
            last_xml_dir: dto.last_xml_dir.clone(),
        }
    }
}

impl From<Memory> for MemoryDTO {
    fn from(memory: Memory) -> Self {
        MemoryDTO {
            invoice_number_column: memory.invoice_number_column,
            good_service_identifier_column: memory.good_service_identifier_column,
            tag_mappings: memory
                .tag_mappings
                .into_iter()
                .map(|(k, m)| {
                    (
                        k,
                        TagMappingDTO {
                            mapped_column: m.mapped_column,
                            default_value: m.default_value,
                        },
                    )
                })
                .collect(),
            last_excel_dir: memory.last_excel_dir,
            last_xml_dir: memory.last_xml_dir,
        }
    }
}
