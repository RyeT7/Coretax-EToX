use std::collections::HashMap;

use crate::domain::entities::tagmapping::TagMapping;

#[derive(Clone, Debug, Default)]
pub struct Memory {
    pub invoice_number_column: String,
    pub good_service_identifier_column: String,
    pub tag_mappings: HashMap<String, TagMapping>,
    pub last_excel_dir: Option<String>,
    pub last_xml_dir: Option<String>,
}
