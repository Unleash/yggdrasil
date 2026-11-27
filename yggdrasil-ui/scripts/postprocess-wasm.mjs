import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const wasmDir = join(__dirname, '..', 'src', 'wasm');
const wasmShim = join(wasmDir, 'yggdrasil_wasm.js');
const envStub = join(wasmDir, 'env.js');

try {
  const original = await readFile(wasmShim, 'utf8');
  const updated = original.replace("import * as __wbg_star0 from 'env';", "import * as __wbg_star0 from './env.js';");

  if (original !== updated) {
    await writeFile(wasmShim, updated, 'utf8');
    console.log('[postprocess-wasm] patched env import');
  }
  try {
    await writeFile(envStub, '// wasm-bindgen expects an `env` import; no browser hooks needed.\nexport {};\n', {
      encoding: 'utf8',
      flag: 'wx',
    });
    console.log('[postprocess-wasm] created env.js stub');
  } catch (error) {
    if ((error)?.code !== 'EEXIST') {
      throw error;
    }
  }
} catch (err) {
  console.warn('[postprocess-wasm] skipped:', err instanceof Error ? err.message : err);
}
