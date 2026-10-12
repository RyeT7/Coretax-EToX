import { ref, computed } from "vue";
import { createSession, sessionId } from "../services/SessionService";
import { open, save } from "@tauri-apps/plugin-dialog";
import { dirname, join } from "@tauri-apps/api/path";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { uploadExcelFile, viewExcelTable } from "../services/ExcelService";
import { getHeaders, mapHeaders, convert, saveXml } from "../services/ParseService";
import {
  defaultMemoryPath,
  loadMemory,
  saveMemory,
  type Memory,
} from "../services/MemoryService";

export interface TagMapping {
  literal: string;
  hierarchical: string;
  derived: boolean; // computed during conversion; not user-editable
  formula: string | null; // shown in place of the inputs on derived tags
  mappedColumn: string | null; // Excel header name or null (use default value)
  defaultValue: string;
}

export interface Step {
  id: number;
  label: string;
}

const steps: Step[] = [
  { id: 1, label: "Upload" },
  { id: 2, label: "Map" },
  { id: 3, label: "Convert" },
];

// --- shared reactive state (module-level singleton) ---
const currentStep = ref(1);

const loadingSession = ref(false);
const sessionError = ref<string | null>(null);
let sessionInitialized = false;

const uploaded = ref(false);
const uploading = ref(false);
const uploadError = ref<string | null>(null);

const headers = ref<string[]>([]);
const tableData = ref<Record<string, string>[]>([]);

const tagMappings = ref<TagMapping[]>([]);
const tin = ref("");
const invoiceNumberColumn = ref("");
const goodServiceIdentifierColumn = ref("");

const converting = ref(false);
const conversionError = ref<string | null>(null);
const conversionSuccess = ref(false);
const savedXmlPath = ref<string | null>(null);

const memoryPath = ref<string | null>(null);
const memory = ref<Memory | null>(null);
const memoryError = ref<string | null>(null);
const memoryLoadFailed = ref(false);

function errMsg(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function applyMemory() {
  const saved = memory.value;
  if (!saved) return;

  const hasHeader = (column: string | null): column is string =>
    !!column && headers.value.includes(column);

  if (hasHeader(saved.invoice_number_column)) {
    invoiceNumberColumn.value = saved.invoice_number_column;
  }
  if (hasHeader(saved.good_service_identifier_column)) {
    goodServiceIdentifierColumn.value = saved.good_service_identifier_column;
  }

  for (const mapping of tagMappings.value) {
    if (mapping.derived) continue;
    const savedMapping = saved.tag_mappings[mapping.hierarchical];
    if (!savedMapping) continue;

    mapping.mappedColumn = hasHeader(savedMapping.mapped_column)
      ? savedMapping.mapped_column
      : null;
    mapping.defaultValue = savedMapping.default_value ?? "";
  }
}

function currentMappings(): Partial<Memory> {
  const tag_mappings = { ...(memory.value?.tag_mappings ?? {}) };
  for (const mapping of tagMappings.value) {
    if (mapping.derived) continue;
    tag_mappings[mapping.hierarchical] = {
      mapped_column: mapping.mappedColumn,
      default_value: mapping.defaultValue,
    };
  }

  return {
    invoice_number_column: invoiceNumberColumn.value,
    good_service_identifier_column: goodServiceIdentifierColumn.value,
    tag_mappings,
  };
}

async function persistMemory(changes: Partial<Memory>) {
  if (!memoryPath.value || memoryLoadFailed.value) return;

  const next: Memory = {
    invoice_number_column: "",
    good_service_identifier_column: "",
    tag_mappings: {},
    last_excel_dir: null,
    last_xml_dir: null,
    ...memory.value,
    ...changes,
  };

  try {
    await saveMemory(memoryPath.value, next);
    memory.value = next;
    memoryError.value = null;
  } catch (error) {
    memoryError.value = errMsg(error);
  }
}

async function loadMemoryFromPath() {
  if (!memoryPath.value) return;

  try {
    memory.value = await loadMemory(memoryPath.value);
    memoryError.value = null;
    memoryLoadFailed.value = false;
  } catch (error) {
    memory.value = null;
    memoryError.value = errMsg(error);
    memoryLoadFailed.value = true;
    return;
  }

  if (!memory.value) await persistMemory({});
}

async function revealMemoryFile() {
  if (!memoryPath.value) return;

  try {
    await revealItemInDir(memoryPath.value);
  } catch (error) {
    memoryError.value = errMsg(error);
  }
}

async function revealXmlFile() {
  if (!savedXmlPath.value) return;

  try {
    await revealItemInDir(savedXmlPath.value);
  } catch (error) {
    conversionError.value = errMsg(error);
  }
}

async function initSession() {
  if (sessionInitialized) return;
  sessionInitialized = true;

  loadingSession.value = true;
  try {
    await createSession();
  } catch (error) {
    sessionError.value = errMsg(error);
  } finally {
    loadingSession.value = false;
  }

  try {
    memoryPath.value = await defaultMemoryPath();
    await loadMemoryFromPath();
  } catch (error) {
    memoryError.value = errMsg(error);
  }
}

async function uploadFile() {
  if (!sessionId.value) {
    uploadError.value = "Session not initialized";
    return;
  }

  uploading.value = true;
  uploadError.value = null;

  try {
    const selected = await open({
      defaultPath: memory.value?.last_excel_dir ?? undefined,
      filters: [{ name: "Excel", extensions: ["xlsx", "xls", "csv"] }],
    });

    if (typeof selected === "string" && selected) {
      await uploadExcelFile(sessionId.value, selected);

      const table = await viewExcelTable(sessionId.value);
      headers.value = table.headers;
      tableData.value = table.data;

      const requiredTags = await getHeaders();
      tagMappings.value = (Array.isArray(requiredTags) ? requiredTags : []).map((tag) => ({
        literal: tag.literal,
        hierarchical: tag.hierarchical,
        derived: tag.derived,
        formula: tag.formula,
        mappedColumn: null,
        defaultValue: "",
      }));

      // Re-uploading invalidates any previous conversion / selections.
      invoiceNumberColumn.value = "";
      goodServiceIdentifierColumn.value = "";
      conversionSuccess.value = false;
      conversionError.value = null;
      savedXmlPath.value = null;
      uploaded.value = true;

      applyMemory();
      await persistMemory({ last_excel_dir: await dirname(selected) });
    }
  } catch (error) {
    uploadError.value = errMsg(error);
  } finally {
    uploading.value = false;
  }
}

async function runConvert() {
  if (!sessionId.value) {
    conversionError.value = "Session not initialized";
    return;
  }

  converting.value = true;
  conversionError.value = null;
  conversionSuccess.value = false;
  savedXmlPath.value = null;

  try {
    await mapHeaders(
      sessionId.value,
      tagMappings.value,
      invoiceNumberColumn.value,
      goodServiceIdentifierColumn.value,
    );
    await convert(sessionId.value, tin.value);
    conversionSuccess.value = true;
  } catch (error) {
    conversionError.value = errMsg(error);
  } finally {
    converting.value = false;
  }
}

async function downloadXml() {
  if (!sessionId.value) {
    conversionError.value = "Session not initialized";
    return;
  }

  try {
    const fileName = `converted-${new Date().toISOString().slice(0, 10)}.xml`;
    const lastXmlDir = memory.value?.last_xml_dir;
    const path = await save({
      defaultPath: lastXmlDir ? await join(lastXmlDir, fileName) : fileName,
      filters: [{ name: "XML", extensions: ["xml"] }],
    });

    if (!path) return; // user cancelled

    await saveXml(sessionId.value, path);
    savedXmlPath.value = path;
    await persistMemory({ last_xml_dir: await dirname(path) });
  } catch (error) {
    conversionError.value = errMsg(error);
  }
}

const mappingsValid = computed(
  () =>
    tin.value.trim() !== "" &&
    invoiceNumberColumn.value !== "" &&
    goodServiceIdentifierColumn.value !== "",
);

const canProceed = computed(() => {
  if (currentStep.value === 1) return uploaded.value;
  if (currentStep.value === 2) return mappingsValid.value;
  return true;
});

function next() {
  if (canProceed.value && currentStep.value < steps.length) {
    if (currentStep.value === 2) void persistMemory(currentMappings());
    currentStep.value++;
  }
}

function back() {
  if (currentStep.value > 1) {
    currentStep.value--;
  }
}

export function useConverter() {
  return {
    steps,
    currentStep,
    loadingSession,
    sessionError,
    uploaded,
    uploading,
    uploadError,
    headers,
    tableData,
    tagMappings,
    tin,
    invoiceNumberColumn,
    goodServiceIdentifierColumn,
    converting,
    conversionError,
    conversionSuccess,
    sessionId,
    savedXmlPath,
    memoryPath,
    memoryError,
    revealMemoryFile,
    revealXmlFile,
    mappingsValid,
    canProceed,
    initSession,
    uploadFile,
    runConvert,
    downloadXml,
    next,
    back,
  };
}
