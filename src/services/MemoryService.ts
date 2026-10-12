import { invoke } from "@tauri-apps/api/core";

export interface MemoryTagMapping {
    mapped_column: string | null;
    default_value: string | null;
}

export interface Memory {
    invoice_number_column: string;
    good_service_identifier_column: string;
    tag_mappings: Record<string, MemoryTagMapping>;
    last_excel_dir: string | null;
    last_xml_dir: string | null;
}

export async function loadMemory(path: string): Promise<Memory | null> {
    return await invoke<Memory | null>("load_memory", {
        path: path,
    });
}

export async function saveMemory(path: string, memory: Memory): Promise<void> {
    return await invoke<void>("save_memory", {
        path: path,
        memory: memory,
    });
}

export async function defaultMemoryPath(): Promise<string> {
    return await invoke<string>("default_memory_path", {});
}
