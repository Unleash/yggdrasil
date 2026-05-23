import initWasm, * as wasmModule from './wasm/yggdrasil_wasm.js';
import { __setWasmMemory } from './wasm/env.js';

export * from './wasm/yggdrasil_wasm.js';

let initialising: Promise<typeof wasmModule> | null = null;

export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module) {
  if (!initialising) {
    initialising = initWasm(input).then((exports) => {
      if (typeof __setWasmMemory === 'function' && exports && typeof (exports as { memory?: WebAssembly.Memory }).memory === 'object') {
        const memory = (exports as { memory?: WebAssembly.Memory }).memory;
        if (memory) {
          __setWasmMemory(() => memory);
        }
      }
      return wasmModule;
    });
  }

  return initialising;
}
