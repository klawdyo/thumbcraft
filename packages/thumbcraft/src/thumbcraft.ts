import { extname } from 'node:path';

import type { IBinaryResolver } from './binary-resolver.ts';
import { InternalError, UnsupportedFormatError } from './errors.ts';
import { errorFromRun } from './exit-codes.ts';
import { type IFileSystem, NodeFileSystem } from './file-system.ts';
import { createDefaultBinaryResolver } from './platform-binary-resolver.ts';
import { type IProcessRunner, NodeProcessRunner } from './process-runner.ts';
import { buildArgs, type RenderOptions } from './render-options.ts';

/** Tempo limite padrão de cada execução do binário, em milissegundos. */
export const DEFAULT_TIMEOUT_MS = 30_000;

/** Dependências do wrapper. Tudo é opcional: o que faltar usa a implementação real. */
export interface IThumbcraftConfig {
  /** Executa o binário. */
  runner?: IProcessRunner;
  /** Acessa o disco (arquivos temporários). */
  fileSystem?: IFileSystem;
  /** Descobre o caminho do binário. */
  binaryResolver?: IBinaryResolver;
  /** Tempo limite de cada execução, em milissegundos (padrão: 30 000). */
  timeoutMs?: number;
}

/** Motor que trata um arquivo: Word, planilha, apresentação ou PDF. */
export type Engine = 'word' | 'grid' | 'deck' | 'pdf';

const ENGINES: readonly string[] = ['word', 'grid', 'deck', 'pdf'];

/** Extensão do arquivo temporário de saída por formato; a do formato decide o que o binário grava. */
const OUTPUT_EXTENSIONS = { png: 'png', jpeg: 'jpg', webp: 'webp' } as const;

/** De onde vem o documento: um arquivo em disco ou bytes em memória (com o nome, para o roteamento). */
type Source = { kind: 'file'; path: string } | { kind: 'buffer'; buffer: Buffer; filename: string };

/**
 * Uma tarefa de miniatura: um documento e a forma de transformá-lo em imagem. Cada chamada de
 * `toBuffer` ou `toFile` executa um processo `thumbcraft` novo, que nunca é reaproveitado
 * (uma falha interna derruba o processo inteiro, por causa do `panic = "abort"`).
 */
export class ThumbcraftJob {
  private readonly source: Source;
  private readonly runner: IProcessRunner;
  private readonly fileSystem: IFileSystem;
  private readonly binaryResolver: IBinaryResolver;
  private readonly timeoutMs: number;

  /**
   * @param source Documento de entrada.
   * @param config Dependências; as omitidas usam as reais.
   */
  constructor(source: Source, config: IThumbcraftConfig = {}) {
    this.source = source;
    this.runner = config.runner ?? new NodeProcessRunner();
    this.fileSystem = config.fileSystem ?? new NodeFileSystem();
    this.binaryResolver = config.binaryResolver ?? createDefaultBinaryResolver();
    this.timeoutMs = config.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  }

  /**
   * Renderiza o documento e devolve os bytes da imagem.
   * @param options Opções de renderização.
   * @throws {ThumbcraftError} Erro tipado conforme o código de saída do binário.
   */
  async toBuffer(options: RenderOptions = {}): Promise<Buffer> {
    const directory = await this.fileSystem.makeTempDir();
    try {
      const output = this.fileSystem.joinPath(directory, `output.${OUTPUT_EXTENSIONS[options.format ?? 'png']}`);
      const input = await this.prepareInput(directory);
      await this.execute(input, output, options);
      return await this.fileSystem.readFile(output);
    } finally {
      await this.fileSystem.removeDir(directory);
    }
  }

  /**
   * Renderiza o documento e grava a imagem em `outputPath`.
   * @param outputPath Caminho do arquivo de saída; a extensão escolhe o formato (a menos que `options.format` o defina).
   * @param options Opções de renderização.
   * @throws {ThumbcraftError} Erro tipado conforme o código de saída do binário.
   */
  async toFile(outputPath: string, options: RenderOptions = {}): Promise<void> {
    if (this.source.kind === 'file') {
      await this.execute(this.source.path, outputPath, options);
      return;
    }
    const directory = await this.fileSystem.makeTempDir();
    try {
      const input = await this.prepareInput(directory);
      await this.execute(input, outputPath, options);
    } finally {
      await this.fileSystem.removeDir(directory);
    }
  }

  /** Devolve o caminho do documento; para bytes em memória, grava um arquivo temporário que mantém a extensão. */
  private async prepareInput(directory: string): Promise<string> {
    if (this.source.kind === 'file') {
      return this.source.path;
    }
    const extension = extname(this.source.filename).toLowerCase();
    const path = this.fileSystem.joinPath(directory, `input${extension}`);
    await this.fileSystem.writeFile(path, this.source.buffer);
    return path;
  }

  /** Executa o binário uma vez e transforma o resultado em erro tipado, se não deu certo. */
  private async execute(input: string, output: string, options: RenderOptions): Promise<void> {
    const binary = this.binaryResolver.resolve();
    const result = await this.runner.run(binary, [input, output, ...buildArgs(options)], this.timeoutMs);
    const error = errorFromRun(result, this.timeoutMs);
    if (error !== null) {
      throw error;
    }
  }
}

/**
 * Pergunta ao binário (`--detect`) qual motor trataria o documento. Não há análise de formato em TypeScript.
 * @returns O motor, ou `null` quando o binário recusa o formato (código 3).
 */
async function runDetect(args: string[], stdin: Buffer | undefined, config: IThumbcraftConfig): Promise<Engine | null> {
  const runner = config.runner ?? new NodeProcessRunner();
  const timeoutMs = config.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const binary = (config.binaryResolver ?? createDefaultBinaryResolver()).resolve();
  const result = await runner.run(binary, args, timeoutMs, stdin);
  const error = errorFromRun(result, timeoutMs);
  if (error instanceof UnsupportedFormatError) {
    return null;
  }
  if (error !== null) {
    throw error;
  }
  const engine = result.stdout.trim();
  if (!ENGINES.includes(engine)) {
    throw new InternalError(`resposta inesperada de 'thumbcraft --detect': '${engine}'`, result.exitCode, result.stderr);
  }
  return engine as Engine;
}

/** Ponto de entrada da API: cria tarefas a partir de um arquivo ou de bytes em memória. */
export class Thumbcraft {
  /**
   * Cria uma tarefa a partir de um arquivo em disco.
   * @param path Caminho do documento.
   * @param config Dependências opcionais (útil em testes).
   */
  static fromFile(path: string, config?: IThumbcraftConfig): ThumbcraftJob {
    return new ThumbcraftJob({ kind: 'file', path }, config);
  }

  /**
   * Cria uma tarefa a partir de bytes em memória.
   * @param buffer Conteúdo do documento.
   * @param filename Nome original do arquivo; a extensão ajuda o binário a escolher o motor.
   * @param config Dependências opcionais (útil em testes).
   */
  static fromBuffer(buffer: Buffer, filename: string, config?: IThumbcraftConfig): ThumbcraftJob {
    return new ThumbcraftJob({ kind: 'buffer', buffer, filename }, config);
  }

  /**
   * Diz qual motor trataria um arquivo em disco.
   * @param path Caminho do documento.
   * @param config Dependências opcionais (útil em testes).
   * @returns O motor, ou `null` quando o formato não é suportado.
   * @throws {ThumbcraftError} Arquivo ilegível ou inexistente (`InvalidFileError`) e qualquer outra falha do processo.
   */
  static detectFile(path: string, config: IThumbcraftConfig = {}): Promise<Engine | null> {
    return runDetect(['--detect', path], undefined, config);
  }

  /**
   * Diz qual motor trataria bytes em memória (enviados pela stdin, sem arquivo temporário).
   * @param buffer Conteúdo do documento.
   * @param filename Nome original; a extensão ajuda quando o conteúdo sozinho não decide.
   * @param config Dependências opcionais (útil em testes).
   * @returns O motor, ou `null` quando o formato não é suportado.
   * @throws {ThumbcraftError} Entrada vazia ou grande demais (`InvalidFileError`) e qualquer outra falha do processo.
   */
  static detect(buffer: Buffer, filename?: string, config: IThumbcraftConfig = {}): Promise<Engine | null> {
    const hint = filename === undefined ? [] : ['--input-name', filename];
    return runDetect(['--detect', '-', ...hint], buffer, config);
  }

  /** Verdadeiro quando algum motor trata o arquivo; os erros que não são de formato não suportado são lançados. */
  static async isValidFile(path: string, config?: IThumbcraftConfig): Promise<boolean> {
    return (await Thumbcraft.detectFile(path, config)) !== null;
  }

  /** Verdadeiro quando algum motor trata os bytes; os erros que não são de formato não suportado são lançados. */
  static async isValid(buffer: Buffer, filename?: string, config?: IThumbcraftConfig): Promise<boolean> {
    return (await Thumbcraft.detect(buffer, filename, config)) !== null;
  }
}
