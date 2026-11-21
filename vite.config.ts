import { existsSync, realpathSync } from 'node:fs';
import { resolve } from 'node:path';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

const wasm = resolve('src/wasm/drum_engine_bg.wasm');
export default defineConfig({
  base: '/drum-vrm/',
  // A routed build directory can put the generated binary outside the workspace.
  server: { fs: { allow: [searchForWorkspaceRoot(process.cwd()), ...(existsSync(wasm) ? [realpathSync(wasm)] : [])] } },
  build: { target: 'es2022', sourcemap: true },
});
