import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, readdir, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { describe, it } from 'node:test';

import { InvalidFileError, UnsupportedFormatError } from './errors.ts';
import { Thumbcraft } from './thumbcraft.ts';

/** Só roda com o binário real indicado em THUMBCRAFT_BIN. */
const skip = process.env.THUMBCRAFT_BIN ? false : 'defina THUMBCRAFT_BIN com o caminho do binário thumbcraft';

const corpus = (relative: string): string => resolve(import.meta.dirname, '../../../corpus', relative);

/** Lê a largura e a altura do cabeçalho IHDR de um PNG. */
function pngSize(bytes: Buffer): { width: number; height: number } {
  assert.deepEqual([...bytes.subarray(0, 4)], [0x89, 0x50, 0x4e, 0x47], 'não é um PNG');
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

describe('real binary', { skip }, () => {
  it('renders a docx to a 120 x 120 PNG', async () => {
    const bytes = await Thumbcraft.fromFile(corpus('docx/01_docx_simples_acentos.docx')).toBuffer({ width: 120, height: 120 });
    assert.deepEqual(pngSize(bytes), { width: 120, height: 120 });
  });

  it('renders from a buffer, keeping the extension for routing', async () => {
    const buffer = await readFile(corpus('pptx/01_titulo_acentos.pptx'));
    const bytes = await Thumbcraft.fromBuffer(buffer, 'slide.pptx').toBuffer({ width: 200, height: 100, fit: 'contain' });
    assert.deepEqual(pngSize(bytes), { width: 200, height: 100 });
  });

  it('writes the image to a file', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'thumbcraft-int-'));
    try {
      const out = join(dir, 'sheet.png');
      await Thumbcraft.fromFile(corpus('xlsx/02-valores-simples.xlsx')).toFile(out, { width: 160 });
      assert.equal(pngSize(await readFile(out)).width, 160);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('maps an unsupported format (.ppt) to UnsupportedFormatError', async () => {
    await assert.rejects(Thumbcraft.fromFile(corpus('pptx/35_legado_formas.ppt')).toBuffer(), UnsupportedFormatError);
  });

  it('maps a damaged file to InvalidFileError', async () => {
    await assert.rejects(Thumbcraft.fromFile(corpus('docx/21_vazio.docx')).toBuffer(), InvalidFileError);
  });
});

describe('real binary: detect', { skip }, () => {
  const bin = process.env.THUMBCRAFT_BIN as string;
  const folders = ['docx', 'xlsx', 'pptx', 'pdf'];

  it('detectFile and detect (buffer) match the text printed by thumbcraft --detect, for the whole corpus', async () => {
    let checked = 0;
    for (const folder of folders) {
      for (const name of await readdir(corpus(folder))) {
        const path = corpus(`${folder}/${name}`);
        const printed = (() => {
          try {
            return execFileSync(bin, ['--detect', path], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
          } catch (error) {
            const status = (error as { status?: number }).status;
            return status === 3 ? null : `erro ${status}`;
          }
        })();
        if (printed?.startsWith('erro')) {
          await assert.rejects(Thumbcraft.detectFile(path), InvalidFileError, name);
        } else {
          assert.equal(await Thumbcraft.detectFile(path), printed, name);
          const bytes = await readFile(path);
          if (bytes.length === 0) {
            // Por contrato, stdin vazia é código 4, enquanto um arquivo vazio ainda é roteado pela extensão.
            await assert.rejects(Thumbcraft.detect(bytes, name), InvalidFileError, `${name} (buffer vazio)`);
          } else {
            assert.equal(await Thumbcraft.detect(bytes, name), printed, `${name} (buffer)`);
          }
        }
        checked += 1;
      }
    }
    assert.ok(checked > 0);
  });

  it('returns null for .ppt and false for isValidFile', async () => {
    assert.equal(await Thumbcraft.detectFile(corpus('pptx/35_legado_formas.ppt')), null);
    assert.equal(await Thumbcraft.isValidFile(corpus('pptx/35_legado_formas.ppt')), false);
  });

  it('detects a PDF as pdf, and a missing file throws InvalidFileError', async () => {
    const pdf = (await readdir(corpus('pdf'))).find((name) => name.endsWith('.pdf'));
    assert.ok(pdf);
    assert.equal(await Thumbcraft.detectFile(corpus(`pdf/${pdf}`)), 'pdf');
    await assert.rejects(Thumbcraft.detectFile(corpus('docx/nao-existe.docx')), InvalidFileError);
  });

  it('uses the name hint for the buffer when the content is silent', async () => {
    assert.equal(await Thumbcraft.detect(Buffer.from('Olá, mundo'), 'nota.txt'), 'word');
    assert.equal(await Thumbcraft.detect(Buffer.from('Olá, mundo')), null);
  });
});
