/** Retângulo de recorte em frações (0 a 1) da página renderizada, aplicado antes do redimensionamento. */
export interface ICropRect {
  /** Distância da borda esquerda. */
  x: number;
  /** Distância do topo. */
  y: number;
  /** Largura do recorte. */
  width: number;
  /** Altura do recorte. */
  height: number;
}

/** Opções de renderização. Cada uma vira uma flag do binário `thumbcraft`; omitida, vale o padrão dele. */
export interface RenderOptions {
  /** Página ou slide a renderizar, a partir de 1 (`--page`; padrão do binário: 1). */
  page?: number;
  /** Escala nativa em px por pt, ou `'auto'` (`--render-scale`; padrão: `'auto'`, cerca de 2x o tamanho pedido). */
  scale?: number | 'auto';
  /** Remove margens em branco ou transparentes (`--trim`; padrão: desligado). */
  trim?: boolean;
  /** Margem mantida pelo trim, em pixels da renderização nativa (`--trim-pad`; padrão: 8). */
  trimPad?: number;
  /** Recorte em frações da página (`--crop`; padrão: nenhum). */
  crop?: ICropRect;
  /** Como a imagem se ajusta à caixa de largura e altura (`--fit`; padrão: `'cover'`). */
  fit?: 'contain' | 'cover';
  /** Âncora do corte ou do enquadramento (`--gravity`; padrão: `'top'` no cover e `'center'` no contain). */
  gravity?: 'top' | 'bottom' | 'left' | 'right' | 'center' | 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  /** Largura final em pixels (`--width`); sozinha, preserva a proporção. */
  width?: number;
  /** Altura final em pixels (`--height`); sozinha, preserva a proporção. */
  height?: number;
  /** Fundo do contain e sob transparência: `'white'` (padrão), `'transparent'` ou `'#RRGGBB'` (`--background`). */
  background?: string;
  /** Filtro de redução (`--filter`; padrão: `'lanczos'`). */
  filter?: 'lanczos' | 'area';
  /** Formato de saída (`--format`; padrão: pela extensão do arquivo de saída; WebP é sem perdas). */
  format?: 'png' | 'jpeg' | 'webp';
  /** Qualidade do JPEG, de 1 a 100 (`--quality`; padrão: 85). */
  quality?: number;
}

/**
 * Converte as opções nas flags do binário, na ordem em que aparecem em `thumbcraft --help`.
 * Valores fora do limite não são conferidos aqui: o binário é a fonte da verdade e responde com
 * código 2 (`UsageError`).
 *
 * @param options Opções de renderização; omitidas, nenhuma flag é gerada.
 * @returns Argumentos para colocar depois de `ENTRADA SAÍDA`.
 */
export function buildArgs(options: RenderOptions = {}): string[] {
  const args: string[] = [];
  if (options.width !== undefined) args.push('-W', String(options.width));
  if (options.height !== undefined) args.push('-H', String(options.height));
  if (options.fit !== undefined) args.push('--fit', options.fit);
  if (options.gravity !== undefined) args.push('--gravity', options.gravity);
  if (options.background !== undefined) args.push('--background', options.background);
  if (options.trim === true) args.push('--trim');
  if (options.trimPad !== undefined) args.push('--trim-pad', String(options.trimPad));
  if (options.crop !== undefined) {
    const { x, y, width, height } = options.crop;
    args.push('--crop', `${x},${y},${width},${height}`);
  }
  if (options.page !== undefined) args.push('--page', String(options.page));
  if (options.scale !== undefined) args.push('--render-scale', String(options.scale));
  if (options.filter !== undefined) args.push('--filter', options.filter);
  if (options.format !== undefined) args.push('--format', options.format);
  if (options.quality !== undefined) args.push('--quality', String(options.quality));
  return args;
}
