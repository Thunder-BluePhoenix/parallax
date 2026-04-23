// Parallax AI Store — BYO-AI Configuration
import { invoke } from "@tauri-apps/api/core";

export type AIProvider = "openai" | "anthropic" | "ollama" | "gemini" | "custom";

export interface AIConfig {
  provider: AIProvider;
  model: string;
  apiKey: string;
  baseUrl?: string;
  temperature: number;
  airGapMode: boolean;
}

const DEFAULT_CONFIG: AIConfig = {
  provider: "ollama",
  model: "llama3",
  apiKey: "",
  baseUrl: "http://localhost:11434",
  temperature: 0.3,
  airGapMode: false
};

function loadConfig(): AIConfig {
  try {
    const raw = localStorage.getItem("parallax:ai_config");
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return { ...DEFAULT_CONFIG };
}

export const aiConfig = $state<AIConfig>(loadConfig());

export function saveAIConfig() {
  localStorage.setItem("parallax:ai_config", JSON.stringify(aiConfig));
  // Notify Go sidecar if needed
}

export const aiStatus = $state<{ busy: boolean; lastError: string | null }>({
  busy: false,
  lastError: null
});

export async function generateTests(requestId: string, responseData: any): Promise<string> {
  aiStatus.busy = true;
  aiStatus.lastError = null;
  try {
    const result = await invoke<string>("ai_generate_tests", {
      config: aiConfig,
      requestId,
      responseData
    });
    return result;
  } catch (err: any) {
    aiStatus.lastError = err.toString();
    throw err;
  } finally {
    aiStatus.busy = false;
  }
}
