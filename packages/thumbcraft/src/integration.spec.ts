import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
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
    const bytes = await Thumbcraft.fromFile(corpus('docx/01_docx_simples_acentos.docx')).toImageBytes({ width: 120, height: 120 });
    assert.deepEqual(pngSize(bytes), { width: 120, height: 120 });
  });

  it('renders from a buffer, keeping the extension for routing', async () => {
    const buffer = await readFile(corpus('pptx/01_titulo_acentos.pptx'));
    const bytes = await Thumbcraft.fromBuffer(buffer, 'slide.pptx').toImageBytes({ width: 200, height: 100, fit: 'contain' });
    assert.deepEqual(pngSize(bytes), { width: 200, height: 100 });
  });

  it('writes the image to a file', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'thumbcraft-int-'));
    try {
      const out = join(dir, 'sheet.png');
      await Thumbcraft.fromFile(corpus('xlsx/02-valores-simples.xlsx')).toImageFile(out, { width: 160 });
      assert.equal(pngSize(await readFile(out)).width, 160);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it('maps an unsupported format (.ppt) to UnsupportedFormatError', async () => {
    await assert.rejects(Thumbcraft.fromFile(corpus('pptx/35_legado_formas.ppt')).toImageBytes(), UnsupportedFormatError);
  });

  it('maps a damaged file to InvalidFileError', async () => {
    await assert.rejects(Thumbcraft.fromFile(corpus('docx/21_vazio.docx')).toImageBytes(), InvalidFileError);
  });
});
