// Monta os pacotes npm (plataformas e principal) em outputs/packages.
// Uso: node tools/build-packages-cli.ts --version X.Y.Z [--binary <plataforma>=<caminho>]... [--out <pasta>]
import { dirname, join } from 'node:path';

import { NodeBuildFileSystem } from './build-file-system.ts';
import { buildAll, parseArgs } from './build-packages.ts';

const mainDir = join(import.meta.dirname, '..');
const packagesDir = dirname(mainDir);

try {
  const built = await buildAll(
    parseArgs(process.argv.slice(2)),
    { mainDir, platformsDir: join(packagesDir, 'platforms'), defaultOut: join(dirname(packagesDir), 'outputs', 'packages') },
    new NodeBuildFileSystem(),
  );
  for (const directory of built) {
    console.log(`pacote montado: ${directory}`);
  }
} catch (error) {
  console.error(`build-packages: ${error instanceof Error ? error.message : String(error)}`);
  process.exitCode = 1;
}
