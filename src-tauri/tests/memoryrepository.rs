use std::{collections::HashMap, path::PathBuf};

use exceltoxml_lib::{
    application::ports::outbound::memoryrepository::MemoryRepository,
    domain::entities::{memory::Memory, tagmapping::TagMapping},
    infrastructure::adapters::outputs::jsonmemoryrepository::JsonMemoryRepository,
};
use uuid::Uuid;

fn temp_path() -> PathBuf {
    std::env::temp_dir().join(format!("exceltoxml-memory-{}.json", Uuid::new_v4()))
}

#[test]
fn missing_file_loads_as_none() {
    let path = temp_path();

    let loaded = JsonMemoryRepository::new()
        .load(path.to_str().unwrap())
        .unwrap();

    assert!(loaded.is_none());
}

#[test]
fn saved_memory_round_trips() {
    let path = temp_path();
    let repository = JsonMemoryRepository::new();

    let memory = Memory {
        invoice_number_column: "Invoice".to_string(),
        good_service_identifier_column: "Item".to_string(),
        tag_mappings: HashMap::from([
            (
                "TaxInvoiceBulk.TIN".to_string(),
                TagMapping::new(Some("Seller".to_string()), None),
            ),
            (
                "TaxInvoiceBulk.ListOfTaxInvoice.TaxInvoice.TrxCode".to_string(),
                TagMapping::new(None, Some("Normal".to_string())),
            ),
        ]),
        last_excel_dir: Some("C:\\invoices".to_string()),
        last_xml_dir: None,
    };

    repository.save(path.to_str().unwrap(), &memory).unwrap();
    let loaded = repository.load(path.to_str().unwrap()).unwrap().unwrap();
    std::fs::remove_file(&path).unwrap();

    assert_eq!(loaded.invoice_number_column, memory.invoice_number_column);
    assert_eq!(
        loaded.good_service_identifier_column,
        memory.good_service_identifier_column
    );
    assert_eq!(loaded.last_excel_dir, memory.last_excel_dir);
    assert_eq!(loaded.last_xml_dir, memory.last_xml_dir);
    assert_eq!(loaded.tag_mappings.len(), 2);

    let tin = &loaded.tag_mappings["TaxInvoiceBulk.TIN"];
    assert_eq!(tin.mapped_column.as_deref(), Some("Seller"));
    assert_eq!(tin.default_value, None);

    let trx = &loaded.tag_mappings["TaxInvoiceBulk.ListOfTaxInvoice.TaxInvoice.TrxCode"];
    assert_eq!(trx.mapped_column, None);
    assert_eq!(trx.default_value.as_deref(), Some("Normal"));
}

#[test]
fn save_replaces_existing_file() {
    let path = temp_path();
    let repository = JsonMemoryRepository::new();

    let first = Memory {
        invoice_number_column: "First".to_string(),
        ..Memory::default()
    };
    let second = Memory {
        invoice_number_column: "Second".to_string(),
        ..Memory::default()
    };

    repository.save(path.to_str().unwrap(), &first).unwrap();
    repository.save(path.to_str().unwrap(), &second).unwrap();
    let loaded = repository.load(path.to_str().unwrap()).unwrap().unwrap();
    let tmp_left = PathBuf::from(format!("{}.tmp", path.display())).exists();
    std::fs::remove_file(&path).unwrap();

    assert_eq!(loaded.invoice_number_column, "Second");
    assert!(!tmp_left);
}

#[test]
fn legacy_tin_field_is_ignored_and_dropped() {
    let path = temp_path();
    let repository = JsonMemoryRepository::new();
    std::fs::write(
        &path,
        r#"{"version": 1, "tin": "12123123", "invoice_number_column": "Invoice"}"#,
    )
    .unwrap();

    let loaded = repository.load(path.to_str().unwrap()).unwrap().unwrap();
    repository.save(path.to_str().unwrap(), &loaded).unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(&path).unwrap();

    assert_eq!(loaded.invoice_number_column, "Invoice");
    assert!(!contents.contains("tin\""));
    assert!(!contents.contains("12123123"));
}

#[test]
fn unknown_version_is_rejected() {
    let path = temp_path();
    std::fs::write(&path, r#"{"version": 99}"#).unwrap();

    let result = JsonMemoryRepository::new().load(path.to_str().unwrap());
    std::fs::remove_file(&path).unwrap();

    assert!(result.is_err());
}

fn temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("exceltoxml-dir-{}", Uuid::new_v4()))
}

#[test]
fn default_location_prefers_writable_install_dir() {
    let install_dir = temp_dir();
    let app_data_dir = temp_dir();
    std::fs::create_dir_all(&install_dir).unwrap();

    let location = JsonMemoryRepository::new()
        .default_location(Some(&install_dir), &app_data_dir)
        .unwrap();
    let leftovers = std::fs::read_dir(&install_dir).unwrap().count();
    std::fs::remove_dir_all(&install_dir).unwrap();

    assert_eq!(PathBuf::from(location), install_dir.join("memory.json"));
    assert_eq!(leftovers, 0);
    assert!(!app_data_dir.exists());
}

#[test]
fn default_location_falls_back_to_app_data_dir() {
    let missing_install_dir = temp_dir();
    let app_data_dir = temp_dir();

    let repository = JsonMemoryRepository::new();
    let without_install = repository.default_location(None, &app_data_dir).unwrap();
    let unwritable_install = repository
        .default_location(Some(&missing_install_dir), &app_data_dir)
        .unwrap();
    let created = app_data_dir.is_dir();
    std::fs::remove_dir_all(&app_data_dir).unwrap();

    assert_eq!(
        PathBuf::from(&without_install),
        app_data_dir.join("memory.json")
    );
    assert_eq!(without_install, unwritable_install);
    assert!(created);
}

#[test]
fn invalid_json_is_rejected() {
    let path = temp_path();
    std::fs::write(&path, "not json").unwrap();

    let result = JsonMemoryRepository::new().load(path.to_str().unwrap());
    std::fs::remove_file(&path).unwrap();

    assert!(result.is_err());
}
