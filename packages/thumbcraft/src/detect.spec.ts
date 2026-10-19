import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { BinaryNotFoundError, InvalidFileError, UsageError } from './errors.ts';
import { FakeProcessRunner, OK_RESULT } from './fakes/fake-process-runner.ts';
import { Thumbcraft } from './thumbcraft.ts';

const BIN = '/opt/thumbcraft/bin/thumbcraft';
const binaryResolver = { resolve: () => BIN };

const printing = (engine: string) => ({ ...OK_RESULT, stdout: `${engine}\n` });

describe('Thumbcraft.detectFile', () => {
  it('asks the binary with --detect and returns the engine it prints', async () => {
    for (const engine of ['word', 'grid', 'deck', 'pdf'] as const) {
      const runner = new FakeProcessRunner(printing(engine));
      assert.equal(await Thumbcraft.detectFile('/docs/a.bin', { runner, binaryResolver }), engine);
      assert.deepEqual(runner.calls[0], { command: BIN, args: ['--detect', '/docs/a.bin'], timeoutMs: 30_000 });
    }
  });

  it('returns null when the binary refuses with code 3 (unsupported format)', async () => {
    const runner = new FakeProcessRunner({ ...OK_RESULT, exitCode: 3, stderr: 'thumbcraft: formato não suportado\n' });
    assert.equal(await Thumbcraft.detectFile('/docs/a.ppt', { runner, binaryResolver }), null);
  });

  it('propagates code 4 as a typed error instead of null', async () => {
    const runner = new FakeProcessRunner({ ...OK_RESULT, exitCode: 4, stderr: 'thumbcraft: não foi possível abrir\n' });
    await assert.rejects(Thumbcraft.detectFile('/docs/missing.docx', { runner, binaryResolver }), InvalidFileError);
  });

  it('propagates any other failure as a typed error', async () => {
    const runner = new FakeProcessRunner({ ...OK_RESULT, exitCode: 2, stderr: 'thumbcraft: uso\n' });
    await assert.rejects(Thumbcraft.detectFile('/docs/a.docx', { runner, binaryResolver }), UsageError);
  });

  it('rejects an engine name it does not know', async () => {
    const runner = new FakeProcessRunner(printing('lixo'));
    await assert.rejects(Thumbcraft.detectFile('/docs/a.docx', { runner, binaryResolver }), /lixo/);
  });

  it('fails with BinaryNotFoundError when the binary cannot be resolved', async () => {
    const runner = new FakeProcessRunner(printing('word'));
    const resolver = {
      resolve: (): string => {
        throw new BinaryNotFoundError('sem binário');
      },
    };
    await assert.rejects(Thumbcraft.detectFile('/docs/a.docx', { runner, binaryResolver: resolver }), BinaryNotFoundError);
  });
});

describe('Thumbcraft.detect (buffer)', () => {
  it('sends the bytes through the stdin with --detect - and the name as a hint', async () => {
    const runner = new FakeProcessRunner(printing('deck'));
    const buffer = Buffer.from('bytes do documento');
    assert.equal(await Thumbcraft.detect(buffer, 'Slides Finais.PPTX', { runner, binaryResolver }), 'deck');
    assert.deepEqual(runner.calls[0]?.args, ['--detect', '-', '--input-name', 'Slides Finais.PPTX']);
    assert.deepEqual(runner.calls[0]?.stdin, buffer);
  });

  it('works without a name', async () => {
    const runner = new FakeProcessRunner(printing('grid'));
    assert.equal(await Thumbcraft.detect(Buffer.from('x'), undefined, { runner, binaryResolver }), 'grid');
    assert.deepEqual(runner.calls[0]?.args, ['--detect', '-']);
  });

  it('returns null for code 3 and propagates code 4', async () => {
    const unsupported = new FakeProcessRunner({ ...OK_RESULT, exitCode: 3 });
    assert.equal(await Thumbcraft.detect(Buffer.from('x'), 'a.odp', { runner: unsupported, binaryResolver }), null);
    const empty = new FakeProcessRunner({ ...OK_RESULT, exitCode: 4, stderr: 'thumbcraft: a stdin está vazia\n' });
    await assert.rejects(Thumbcraft.detect(Buffer.alloc(0), 'a.docx', { runner: empty, binaryResolver }), InvalidFileError);
  });
});

describe('Thumbcraft.isValidFile and isValid', () => {
  it('are true when an engine handles the file and false on null', async () => {
    const ok = new FakeProcessRunner(printing('word'));
    assert.equal(await Thumbcraft.isValidFile('/docs/a.docx', { runner: ok, binaryResolver }), true);
    assert.equal(await Thumbcraft.isValid(Buffer.from('x'), 'a.docx', { runner: ok, binaryResolver }), true);
    const no = new FakeProcessRunner({ ...OK_RESULT, exitCode: 3 });
    assert.equal(await Thumbcraft.isValidFile('/docs/a.ppt', { runner: no, binaryResolver }), false);
    assert.equal(await Thumbcraft.isValid(Buffer.from('x'), 'a.ppt', { runner: no, binaryResolver }), false);
  });

  it('still throws on errors other than unsupported format', async () => {
    const bad = new FakeProcessRunner({ ...OK_RESULT, exitCode: 4 });
    await assert.rejects(Thumbcraft.isValidFile('/docs/missing.docx', { runner: bad, binaryResolver }), InvalidFileError);
  });
});
