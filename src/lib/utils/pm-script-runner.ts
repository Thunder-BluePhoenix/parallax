// Parallax pm.* Script Runner
// Postman-compatible API surface executed via sandboxed Function()

import type { ResponseState, TestResult } from "../stores/app.svelte";

// ── pm object factory ─────────────────────────────────────────────────────────
function buildPm(env: Record<string, string>, response?: ResponseState) {
  const tests: TestResult[] = [];
  const collectedLogs: string[] = [];
  const visualizerState: { template: string | null; data: any | null } = { template: null, data: null };

  const pmEnv = {
    get: (key: string) => env[key] ?? undefined,
    set: (key: string, val: string) => { env[key] = String(val); },
    unset: (key: string) => { delete env[key]; },
    has: (key: string) => key in env,
    toObject: () => ({ ...env }),
  };

  const pmResponse = response
    ? {
        code: response.status,
        status: response.statusText,
        headers: {
          get: (key: string) =>
            Object.entries(response.headers).find(([k]) => k.toLowerCase() === key.toLowerCase())?.[1],
        },
        responseTime: response.timing.totalMs,
        responseSize: response.sizeBytes,
        text: () => response.body.raw,
        json: () => {
          if (response.body.json) return response.body.json;
          try { return JSON.parse(response.body.raw); } catch { return null; }
        },
      }
    : null;

  const pm = {
    environment: pmEnv,
    globals:     pmEnv, // share same store for simplicity
    variables:   pmEnv,
    response:    pmResponse,
    visualizer: {
      set: (template: string, data?: any) => {
        visualizerState.template = template;
        visualizerState.data = data ?? null;
      },
    },

    test: (name: string, fn: () => void) => {
      try {
        fn();
        tests.push({ name, passed: true });
      } catch (e: any) {
        tests.push({ name, passed: false, error: e?.message ?? String(e) });
      }
    },

    expect: (actual: any) => ({
      to: {
        equal:      (expected: any)  => { if (actual !== expected) throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`); },
        eql:        (expected: any)  => { if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error(`Deep equal failed`); },
        be:         {
          ok:       () => { if (!actual) throw new Error(`Expected truthy, got ${actual}`); },
          above:    (n: number) => { if (actual <= n) throw new Error(`${actual} is not above ${n}`); },
          below:    (n: number) => { if (actual >= n) throw new Error(`${actual} is not below ${n}`); },
          a:        (type: string) => { if (typeof actual !== type) throw new Error(`Expected type ${type}, got ${typeof actual}`); },
          string:   () => { if (typeof actual !== "string") throw new Error(`Expected string`); },
          number:   () => { if (typeof actual !== "number") throw new Error(`Expected number`); },
          boolean:  () => { if (typeof actual !== "boolean") throw new Error(`Expected boolean`); },
          null:     () => { if (actual !== null) throw new Error(`Expected null`); },
          undefined:() => { if (actual !== undefined) throw new Error(`Expected undefined`); },
          oneOf:    (arr: any[]) => { if (!arr.includes(actual)) throw new Error(`${actual} not in [${arr.join(",")}]`); },
        },
        include:    (sub: any)  => { if (!String(actual).includes(String(sub))) throw new Error(`"${actual}" does not include "${sub}"`); },
        have: {
          property: (key: string) => { if (!(key in (actual ?? {}))) throw new Error(`Missing property "${key}"`); },
          length:   (n: number)   => { if (actual?.length !== n) throw new Error(`Expected length ${n}, got ${actual?.length}`); },
          status:   (code: number) => { if (pm.response?.code !== code) throw new Error(`Expected status ${code}, got ${pm.response?.code}`); },
          header:   (name: string) => { if (!pm.response?.headers.get(name)) throw new Error(`Missing header "${name}"`); },
        },
        not: {
          equal:    (v: any) => { if (actual === v) throw new Error(`Expected not ${JSON.stringify(v)}`); },
          include:  (sub: any) => { if (String(actual).includes(String(sub))) throw new Error(`Did not expect "${sub}"`); },
        },
      },
    }),
  };

  return { pm, tests, logs: collectedLogs, env, visualizerState };
}

// ── Pre-request script ─────────────────────────────────────────────────────────
export async function runPreRequestScript(
  script: string,
  env: Record<string, string>,
): Promise<void> {
  const { pm } = buildPm(env);
  const fn = new Function("pm", "console", `"use strict";\n${script}`);
  const fakeConsole = { log: () => {}, warn: () => {}, error: () => {} };
  fn(pm, fakeConsole);
  // env is mutated in-place via pm.environment.set
}

// ── Test script ────────────────────────────────────────────────────────────────
export async function runTestScript(
  script: string,
  response: ResponseState,
  env: Record<string, string>,
): Promise<{ tests: TestResult[]; visualizer: { template: string | null; data: any | null } }> {
  const { pm, tests, visualizerState } = buildPm(env, response);
  const fn = new Function("pm", "console", `"use strict";\n${script}`);
  const fakeConsole = { log: () => {}, warn: () => {}, error: () => {} };
  fn(pm, fakeConsole);
  return { tests, visualizer: visualizerState };
}
