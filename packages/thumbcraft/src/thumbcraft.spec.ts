import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { BinaryNotFoundError, InvalidFileError, TimeoutError } from './errors.ts';
import { FakeFileSystem } from './fakes/fake-file-system.ts';
import { FakeProcessRunner, OK_RESULT } from './fakes/fake-process-runner.ts';
import { EnvBinaryResolver } from './binary-resolver.ts';
import { Thumbcraft } from './thumbcraft.ts';

const BIN = '/opt/thumbcraft/bin/thumbcraft';
const PNG = Buffer.from([0x89, 0x50, 0x4e, 0x47]);

/** Monta um cenário em que o binário "grava" o arquivo de saída (o segundo argumento). */
function scenario(result = OK_RESULT) {
  const fileSystem = new FakeFileSystem();
  const runner = new FakeProcessRunner(result, async (call) => {
    if (result.exitCode === 0 && !result.timedOut) {
      await fileSystem.writeFile(call.args[1] as string, PNG);
    }
  });
  const config = { runner, fileSystem, binaryResolver: { resolve: () => BIN } };
  return { fileSystem, runner, config };
}

describe('Thumbcraft.fromFile', () => {
  it('toFile runs the binary once, straight from input to output, with no temp dir', async () => {
    const { fileSystem, runner, config } = scenario();
    await Thumbcraft.fromFile('/docs/report.docx', config).toFile('/out/report.png', { width: 120, height: 120 });
    assert.equal(runner.calls.length, 1);
    assert.deepEqual(runner.calls[0], {
      command: BIN,
      args: ['/docs/report.docx', '/out/report.png', '-W', '120', '-H', '120'],
      timeoutMs: 30_000,
    });
    assert.equal(fileSystem.createdTempDirs.length, 0);
  });

  it('toBuffer renders to a temp file, returns its bytes and removes the temp dir', async () => {
    const { fileSystem, runner, config } = scenario();
    const bytes = await Thumbcraft.fromFile('/docs/report.docx', config).toBuffer({ page: 2 });
    assert.deepEqual(bytes, PNG);
    assert.deepEqual(runner.calls[0]?.args, ['/docs/report.docx', '/tmp/thumbcraft-fake-1/output.png', '--page', '2']);
    assert.equal(fileSystem.tempDirs.size, 0);
  });

  it('picks the temp file extension from the requested format', async () => {
    const { runner, config } = scenario();
    await Thumbcraft.fromFile('/docs/a.docx', config).toBuffer({ format: 'jpeg' });
    assert.deepEqual(runner.calls[0]?.args, ['/docs/a.docx', '/tmp/thumbcraft-fake-1/output.jpg', '--format', 'jpeg']);
  });
});

describe('Thumbcraft.fromBuffer', () => {
  it('writes the buffer to a temp input that keeps the extension, then renders it', async () => {
    const { fileSystem, runner, config } = scenario();
    const input = Buffer.from('conteudo do documento');
    let seenInput: Buffer | undefined;
    const spy = new FakeProcessRunner(OK_RESULT, async (call) => {
      seenInput = await fileSystem.readFile(call.args[0] as string);
      await fileSystem.writeFile(call.args[1] as string, PNG);
    });
    const bytes = await Thumbcraft.fromBuffer(input, 'Planilha Final.XLSX', { ...config, runner: spy }).toBuffer();
    assert.deepEqual(bytes, PNG);
    assert.deepEqual(spy.calls[0]?.args, ['/tmp/thumbcraft-fake-1/input.xlsx', '/tmp/thumbcraft-fake-1/output.png']);
    assert.deepEqual(seenInput, input);
    assert.equal(fileSystem.tempDirs.size, 0);
    assert.equal(runner.calls.length, 0);
  });

  it('toFile writes the image to the requested path', async () => {
    const { fileSystem, runner, config } = scenario();
    const spy = new FakeProcessRunner(OK_RESULT, async (call) => {
      await fileSystem.writeFile(call.args[1] as string, PNG);
    });
    await Thumbcraft.fromBuffer(Buffer.from('x'), 'a.pptx', { ...config, runner: spy }).toFile('/out/slide.png', { width: 800 });
    assert.deepEqual(spy.calls[0]?.args, ['/tmp/thumbcraft-fake-1/input.pptx', '/out/slide.png', '-W', '800']);
    assert.equal(fileSystem.tempDirs.size, 0);
    assert.equal(runner.calls.length, 0);
  });

  it('uses no extension when the file name has none', async () => {
    const { runner, config } = scenario();
    await Thumbcraft.fromBuffer(Buffer.from('x'), 'documento', config).toBuffer();
    assert.equal(runner.calls[0]?.args[0], '/tmp/thumbcraft-fake-1/input');
  });
});

describe('errors and cleanup', () => {
  it('turns exit code 4 into InvalidFileError and still removes the temp dir', async () => {
    const { fileSystem, config } = scenario({ exitCode: 4, signal: null, stdout: '', stderr: 'thumbcraft: arquivo inválido\n', timedOut: false });
    await assert.rejects(Thumbcraft.fromBuffer(Buffer.from('x'), 'a.docx', config).toBuffer(), (error: unknown) => {
      assert.ok(error instanceof InvalidFileError);
      assert.equal(error.message, 'arquivo inválido');
      return true;
    });
    assert.equal(fileSystem.tempDirs.size, 0);
  });

  it('turns a timeout into TimeoutError', async () => {
    const { config } = scenario({ exitCode: null, signal: 'SIGKILL', stdout: '', stderr: '', timedOut: true });
    await assert.rejects(Thumbcraft.fromFile('/docs/a.docx', config).toFile('/out/a.png'), TimeoutError);
  });

  it('passes the configured timeout to the runner', async () => {
    const { runner, config } = scenario();
    await Thumbcraft.fromFile('/docs/a.docx', { ...config, timeoutMs: 5_000 }).toFile('/out/a.png');
    assert.equal(runner.calls[0]?.timeoutMs, 5_000);
  });

  it('runs one process per call, never reusing it', async () => {
    const { runner, config } = scenario();
    const job = Thumbcraft.fromFile('/docs/a.docx', config);
    await job.toBuffer();
    await job.toBuffer();
    assert.equal(runner.calls.length, 2);
  });

  it('fails with BinaryNotFoundError when the binary cannot be resolved', async () => {
    const { config } = scenario();
    const binaryResolver = new EnvBinaryResolver({});
    await assert.rejects(Thumbcraft.fromFile('/docs/a.docx', { ...config, binaryResolver }).toFile('/out/a.png'), BinaryNotFoundError);
  });
});

describe('EnvBinaryResolver', () => {
  it('reads THUMBCRAFT_BIN', () => {
    assert.equal(new EnvBinaryResolver({ THUMBCRAFT_BIN: '/x/thumbcraft' }).resolve(), '/x/thumbcraft');
  });

  it('throws BinaryNotFoundError, with a hint, when the variable is empty or missing', () => {
    assert.throws(() => new EnvBinaryResolver({}).resolve(), BinaryNotFoundError);
    assert.throws(() => new EnvBinaryResolver({ THUMBCRAFT_BIN: '' }).resolve(), /THUMBCRAFT_BIN/);
  });
});
