use std::sync::{Arc, Mutex};

use crate::{
    application::services::{
        convertservice::ConvertService, createsessionservice::CreateSessionService,
        getxmlservice::GetXmlService, mapheadersservice::MapHeadersService,
        memoryservice::MemoryService, uploadexcelservice::UploadExcelService,
        viewexcelservice::ViewExcelService, viewheadersservice::ViewHeadersService,
    },
    infrastructure::adapters::outputs::{
        calamineexcelreader::CalamineExcelReader, customxmlwriter::CustomXMLWriter,
        jsonmemoryrepository::JsonMemoryRepository, taurisessionrepository::TauriSessionRepository,
        uuidv4generator::Uuidv4Generator,
    },
    state::appstate::AppState,
};

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // infrastructure
    let session_repository = Arc::new(TauriSessionRepository::new());
    let excel_reader = Arc::new(CalamineExcelReader::new());
    let id_generator = Arc::new(Uuidv4Generator::new());
    let xml_writer = Arc::new(Mutex::new(CustomXMLWriter::new()));
    let memory_repository = Arc::new(JsonMemoryRepository::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            // application
            upload_excel_use_case: Arc::new(UploadExcelService::new(
                excel_reader.clone(),
                session_repository.clone(),
            )),
            create_session_use_case: Arc::new(CreateSessionService::new(
                session_repository.clone(),
                id_generator.clone(),
            )),
            map_headers_use_case: Arc::new(MapHeadersService::new(session_repository.clone())),
            view_excel_use_case: Arc::new(ViewExcelService::new(session_repository.clone())),
            view_headers_use_case: Arc::new(ViewHeadersService::new()),
            convert_use_case: Arc::new(Mutex::new(ConvertService::new(
                session_repository.clone(),
                xml_writer.clone(),
            ))),
            get_xml_use_case: Arc::new(GetXmlService::new(session_repository.clone())),
            memory_use_case: Arc::new(MemoryService::new(memory_repository.clone())),
        })
        .invoke_handler(tauri::generate_handler![
            // Excel Controller
            infrastructure::adapters::inputs::excelcontroller::load_excel,
            infrastructure::adapters::inputs::excelcontroller::view_excel,
            // Session Controller
            infrastructure::adapters::inputs::sessioncontroller::create_session,
            // Parser Controller
            infrastructure::adapters::inputs::parsercontroller::view_headers,
            // Map Headers Controller
            infrastructure::adapters::inputs::mapheaderscontroller::map_headers,
            // Convert Controller
            infrastructure::adapters::inputs::convertcontroller::convert,
            // Get XML Controller
            infrastructure::adapters::inputs::getxmlcontroller::get_xml,
            infrastructure::adapters::inputs::getxmlcontroller::save_xml,
            // Memory Controller
            infrastructure::adapters::inputs::memorycontroller::load_memory,
            infrastructure::adapters::inputs::memorycontroller::save_memory,
            infrastructure::adapters::inputs::memorycontroller::default_memory_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
