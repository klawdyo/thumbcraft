import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm', 'cjs'],
  dts: true,
  clean: true,
  platform: 'node',
  target: 'node20',
  tsconfig: 'tsconfig.build.json',
  // `import.meta.url` (usado para achar o pacote de plataforma) vira `__filename` no CJS.
  shims: true,
});
