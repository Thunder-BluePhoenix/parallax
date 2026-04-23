// Parallax AI Store — BYO-AI Configuration & Actions
import { invoke } from "@tauri-apps/api/core";
import { activeRequest, activeEnvironment } from "./app.svelte";

export type AIProvider = "openai" | "anthropic" | "ollama" | "gemini" | "custom";

export interface AIConfig {
  provider: AIProvider;
  model: string;
  apiKey: string;
  baseUrl?: string;
  temperature: number;
  airGapMode: boolean;
}

export interface AIFix {
  type: "header" | "body" | "url" | "auth" | "env";
  description: string;
  patch: Record<string, unknown>;
}

export interface AIRepairResult {
  diagnosis: string;
  fixes: AIFix[];
  priority: "high" | "medium" | "low";
}

const DEFAULT_CONFIG: AIConfig = {
  provider: "ollama",
  model: "llama3",
  apiKey: "",
  baseUrl: "http://localhost:11434",
  temperature: 0.3,
  airGapMode: false,
};

function loadConfig(): AIConfig {
  try {
    const raw = localStorage.getItem("parallax:ai_config");
    if (raw) return { ...DEFAULT_CONFIG, ...JSON.parse(raw) };
  } catch { /* ignore */ }
  return { ...DEFAULT_CONFIG };
}

export const aiConfig = $state<AIConfig>(loadConfig());

export function saveAIConfig() {
  localStorage.setItem("parallax:ai_config", JSON.stringify(aiConfig));
}

export const aiStatus = $state<{
  busy: boolean;
  lastError: string | null;
  repairResult: AIRepairResult | null;
  generatedScript: string | null;
  generatedCollection: string | null;
}>({
  busy: false,
  lastError: null,
  repairResult: null,
  generatedScript: null,
  generatedCollection: null,
});

// ── Generate Tests ────────────────────────────────────────────────────────────

export async function generateTests(requestId: string, response: any): Promise<{ js: string; yaml: string }> {
  if (aiConfig.airGapMode && aiConfig.provider !== "ollama") {
    throw new Error("Air-gap mode is enabled. Only Ollama (local) is allowed.");
  }
  aiStatus.busy = true;
  aiStatus.lastError = null;
  try {
    const result = await invoke<{ js: string; yaml: string }>("ai_generate_tests", {
      config: aiConfig,
      requestId,
      method:          activeRequest.method,
      url:             activeRequest.url,
      responseBody:    typeof response.body?.json !== "undefined"
                         ? JSON.stringify(response.body.json)
                         : (response.body?.text ?? ""),
      responseStatus:  response.status,
      responseHeaders: response.headers ?? {},
    });
    return result;
  } catch (err: any) {
    aiStatus.lastError = err.toString();
    throw err;
  } finally {
    aiStatus.busy = false;
  }
}

// ── Repair Request ────────────────────────────────────────────────────────────

export async function repairRequest(response: any): Promise<AIRepairResult> {
  if (aiConfig.airGapMode && aiConfig.provider !== "ollama") {
    throw new Error("Air-gap mode is enabled. Only Ollama (local) is allowed.");
  }
  aiStatus.busy = true;
  aiStatus.lastError = null;
  aiStatus.repairResult = null;
  try {
    const envKeys = Object.keys(activeEnvironment.variables ?? {});
    const result = await invoke<AIRepairResult>("ai_repair_request", {
      config:          aiConfig,
      method:          activeRequest.method,
      url:             activeRequest.url,
      requestHeaders:  activeRequest.headers ?? {},
      requestBody:     activeRequest.bodyContent ?? "",
      responseStatus:  response.status,
      responseBody:    typeof response.body?.json !== "undefined"
                         ? JSON.stringify(response.body.json)
                         : (response.body?.text ?? ""),
      envKeys,
    });
    aiStatus.repairResult = result;
    return result;
  } catch (err: any) {
    aiStatus.lastError = err.toString();
    throw err;
  } finally {
    aiStatus.busy = false;
  }
}

// ── Generate Script ───────────────────────────────────────────────────────────

export async function generateScript(scriptType: "pre-request" | "test", userPrompt: string): Promise<string> {
  if (aiConfig.airGapMode && aiConfig.provider !== "ollama") {
    throw new Error("Air-gap mode is enabled. Only Ollama (local) is allowed.");
  }
  aiStatus.busy = true;
  aiStatus.lastError = null;
  aiStatus.generatedScript = null;
  try {
    const script = await invoke<string>("ai_generate_script", {
      config:     aiConfig,
      scriptType,
      userPrompt,
      method:     activeRequest.method,
      url:        activeRequest.url,
    });
    aiStatus.generatedScript = script;
    return script;
  } catch (err: any) {
    aiStatus.lastError = err.toString();
    throw err;
  } finally {
    aiStatus.busy = false;
  }
}

// ── Create Collection ─────────────────────────────────────────────────────────

export async function createCollectionFromDescription(description: string): Promise<string> {
  if (aiConfig.airGapMode && aiConfig.provider !== "ollama") {
    throw new Error("Air-gap mode is enabled. Only Ollama (local) is allowed.");
  }
  aiStatus.busy = true;
  aiStatus.lastError = null;
  aiStatus.generatedCollection = null;
  try {
    const yaml = await invoke<string>("ai_create_collection", {
      config: aiConfig,
      description,
    });
    aiStatus.generatedCollection = yaml;
    return yaml;
  } catch (err: any) {
    aiStatus.lastError = err.toString();
    throw err;
  } finally {
    aiStatus.busy = false;
  }
}
