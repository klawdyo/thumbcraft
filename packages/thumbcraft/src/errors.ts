/**
 * Erro base de tudo que o wrapper lança. Guarda o código de saída do binário (quando houve um)
 * e a stderr original, para diagnóstico.
 */
export class ThumbcraftError extends Error {
  /** Código de saída do binário, ou `null` quando o processo não terminou com um código. */
  readonly exitCode: number | null;
  /** Texto bruto da stderr do binário (as mensagens dele são em português). */
  readonly stderr: string;

  /**
   * @param message Mensagem legível, já sem o prefixo `thumbcraft:`.
   * @param exitCode Código de saída do binário, se houver.
   * @param stderr Texto bruto da stderr.
   */
  constructor(message: string, exitCode: number | null = null, stderr = '') {
    super(message);
    this.name = new.target.name;
    this.exitCode = exitCode;
    this.stderr = stderr;
  }
}

/** Uso incorreto da linha de comando: opção inválida ou valor fora do limite (código 2). */
export class UsageError extends ThumbcraftError {}

/** Formato não suportado, como `.ppt`, `.odp` ou extensão desconhecida (código 3). */
export class UnsupportedFormatError extends ThumbcraftError {}

/** Arquivo inválido ou falha ao abrir ou renderizar, incluindo página inexistente (código 4). */
export class InvalidFileError extends ThumbcraftError {}

/** Erro interno do binário, incluindo falha ao gravar a saída (código 5). */
export class InternalError extends ThumbcraftError {}

/**
 * O processo terminou de forma inesperada: código fora de 0 a 5 (por exemplo 134, de `panic = "abort"`)
 * ou morte por sinal.
 */
export class ProcessCrashError extends ThumbcraftError {}

/** O binário passou do tempo limite e foi encerrado à força. */
export class TimeoutError extends ThumbcraftError {}

/** Não foi possível encontrar ou iniciar o binário `thumbcraft`. */
export class BinaryNotFoundError extends ThumbcraftError {}
