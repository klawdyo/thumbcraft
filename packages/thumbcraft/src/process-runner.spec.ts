import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { BinaryNotFoundError } from './errors.ts';
import { NodeProcessRunner } from './process-runner.ts';

/** Executa um trecho de JavaScript em um processo Node novo. */
const runScript = (script: string, timeoutMs = 10_000) => new NodeProcessRunner().run(process.execPath, ['-e', script], timeoutMs);

describe('NodeProcessRunner', () => {
  it('returns exit code 0 for a process that succeeds', async () => {
    const result = await runScript('process.exit(0)');
    assert.deepEqual(result, { exitCode: 0, signal: null, stderr: '', timedOut: false });
  });

  it('returns the exit code and the stderr text of a failing process', async () => {
    const result = await runScript('console.error("thumbcraft: arquivo inválido"); process.exit(4)');
    assert.equal(result.exitCode, 4);
    assert.equal(result.stderr.trim(), 'thumbcraft: arquivo inválido');
    assert.equal(result.timedOut, false);
  });

  it('kills a process that passes the time limit and reports the timeout', async () => {
    const started = Date.now();
    const result = await runScript('setTimeout(() => {}, 60000)', 300);
    assert.equal(result.timedOut, true);
    assert.ok(Date.now() - started < 10_000, 'deveria encerrar logo depois do limite');
  });

  it('keeps only a bounded amount of stderr', async () => {
    const result = await runScript('process.stderr.write("x".repeat(500000)); process.exit(5)');
    assert.equal(result.exitCode, 5);
    assert.ok(result.stderr.length <= 64 * 1024);
  });

  it('does not use a shell: arguments with spaces and quotes arrive intact', async () => {
    const runner = new NodeProcessRunner();
    const result = await runner.run(process.execPath, ['-e', 'console.error(process.argv[1]); process.exit(0)', 'a "b" c & d'], 10_000);
    assert.equal(result.stderr.trim(), 'a "b" c & d');
  });

  it('rejects with BinaryNotFoundError when the executable does not exist', async () => {
    await assert.rejects(new NodeProcessRunner().run('/caminho/que/nao/existe/thumbcraft', [], 1_000), BinaryNotFoundError);
  });
});
