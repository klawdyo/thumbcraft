// Processo descartável: gera a miniatura da primeira página com pdf-parse e sai.
// Usado no cenário COLD (inclui start do Node e carga dos módulos).
// Uso: node pdfparse-one.ts ENTRADA.pdf SAIDA.png LARGURA
import { readFileSync, writeFileSync } from 'node:fs';
import { PDFParse } from 'pdf-parse';

const [input, output, width] = process.argv.slice(2);
const buf = readFileSync(input);
const parser = new PDFParse({ data: buf });
try {
  // Mesma chamada do código do usuário, com desiredWidth para igualar a largura de saída.
  const thumb = await parser.getScreenshot({ first: 1, desiredWidth: Number(width) });
  const data = thumb.pages[0]?.data;
  if (!data) throw new Error('Não foi possível extrair a imagem da primeira página do PDF.');
  writeFileSync(output, data);
} finally {
  await parser.destroy();
}
