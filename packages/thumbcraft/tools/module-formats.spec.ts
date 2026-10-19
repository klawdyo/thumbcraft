import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it } from 'node:test';

const MAIN_DIR = join(import.meta.dirname, '..');
const BUILT = existsSync(join(MAIN_DIR, 'dist', 'index.cjs')) && existsSync(join(MAIN_DIR, 'dist', 'index.mjs'));

/** Roda um script no Node, dentro do pacote, e devolve o que ele imprimiu. */
function run(args: string[]): string {
  return execFileSync(process.execPath, args, { cwd: MAIN_DIR, encoding: 'utf8' }).trim();
}

const PROBE = 'console.log(JSON.stringify([typeof m.Thumbcraft.fromFile, typeof m.PlatformBinaryResolver, Object.keys(m).sort()]))';

describe('dist formats', { skip: BUILT ? false : 'run npm run build first' }, () => {
  it('loads through require() with the package name', () => {
    const output = run(['-e', `const m = require('thumbcraft'); ${PROBE}`]);
    const [fromFile, resolver] = JSON.parse(output) as string[];
    assert.equal(fromFile, 'function');
    assert.equal(resolver, 'function');
  });

  it('loads through import with the package name', () => {
    const output = run(['--input-type=module', '-e', `import * as m from 'thumbcraft'; ${PROBE}`]);
    const [fromFile, resolver] = JSON.parse(output) as string[];
    assert.equal(fromFile, 'function');
    assert.equal(resolver, 'function');
  });

  it('exposes the same API in both formats', () => {
    const keys = 'JSON.stringify(Object.keys(m).filter((k) => k !== "default").sort())';
    const cjs = run(['-e', `const m = require('thumbcraft'); console.log(${keys})`]);
    const esm = run(['--input-type=module', '-e', `import * as m from 'thumbcraft'; console.log(${keys})`]);
    assert.deepEqual(JSON.parse(cjs), JSON.parse(esm));
  });

  it('resolves the platform package from the CJS build (the import.meta.url shim)', () => {
    const output = run(['-e', "const { PlatformBinaryResolver } = require('thumbcraft'); try { new PlatformBinaryResolver().resolve(); console.log('found'); } catch (e) { console.log(e.name); }"]);
    assert.match(output, /^(found|BinaryNotFoundError)$/);
  });
});
