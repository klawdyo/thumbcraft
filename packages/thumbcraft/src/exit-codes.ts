import {
  InternalError,
  InvalidFileError,
  ProcessCrashError,
  type ThumbcraftError,
  TimeoutError,
  UnsupportedFormatError,
  UsageError,
} from './errors.ts';
import type { IRunResult } from './process-runner.ts';

/** Mensagem usada quando o binário falhou sem escrever nada na stderr. */
const DEFAULT_MESSAGES: Record<number, string> = {
  2: 'uso incorreto da linha de comando',
  3: 'formato de arquivo não suportado',
  4: 'arquivo inválido ou falha ao abrir ou renderizar',
  5: 'erro interno do thumbcraft',
};

/** Tira o prefixo `thumbcraft:` e os espaços das pontas da stderr. */
function cleanMessage(stderr: string): string {
  return stderr.replace(/^\s*thumbcraft:\s*/, '').trim();
}

/**
 * Traduz o resultado de uma execução em um erro tipado, ou `null` se deu certo.
 *
 * Códigos 2, 3, 4 e 5 são os documentados pelo binário. Qualquer outro status (por exemplo 134, de
 * `panic = "abort"`, ou a morte por sinal) vira {@link ProcessCrashError}, e o estouro de tempo vira
 * {@link TimeoutError}, não importa o código.
 *
 * @param result Resultado da execução.
 * @param timeoutMs Tempo limite usado, para a mensagem do estouro.
 */
export function errorFromRun(result: IRunResult, timeoutMs: number): ThumbcraftError | null {
  if (result.timedOut) {
    return new TimeoutError(`o thumbcraft excedeu o tempo limite de ${timeoutMs} ms`, result.exitCode, result.stderr);
  }
  if (result.exitCode === 0) {
    return null;
  }
  const { exitCode, stderr } = result;
  const message = cleanMessage(stderr);
  switch (exitCode) {
    case 2:
      return new UsageError(message || (DEFAULT_MESSAGES[2] as string), exitCode, stderr);
    case 3:
      return new UnsupportedFormatError(message || (DEFAULT_MESSAGES[3] as string), exitCode, stderr);
    case 4:
      return new InvalidFileError(message || (DEFAULT_MESSAGES[4] as string), exitCode, stderr);
    case 5:
      return new InternalError(message || (DEFAULT_MESSAGES[5] as string), exitCode, stderr);
    default: {
      const how = exitCode === null ? `sinal ${result.signal ?? 'desconhecido'}` : `código ${exitCode}`;
      return new ProcessCrashError(`o processo thumbcraft terminou de forma inesperada (${how})`, exitCode, stderr);
    }
  }
}
