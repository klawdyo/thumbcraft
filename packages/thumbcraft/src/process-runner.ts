import { spawn } from 'node:child_process';

import { BinaryNotFoundError, InternalError } from './errors.ts';

/** Resultado de uma execução do binário. */
export interface IRunResult {
  /** Código de saída, ou `null` quando o processo morreu por sinal. */
  exitCode: number | null;
  /** Sinal que encerrou o processo, quando houve. */
  signal: string | null;
  /** Texto da stderr (limitado a alguns KB). */
  stderr: string;
  /** Verdadeiro quando o processo foi morto por passar do tempo limite. */
  timedOut: boolean;
}

/** Porta para executar um processo. A implementação real inicia um processo; os testes usam um falso. */
export interface IProcessRunner {
  /**
   * Executa um processo novo e espera o fim dele.
   * @param command Caminho do executável.
   * @param args Argumentos, sem passar por shell.
   * @param timeoutMs Tempo limite; passado dele o processo é morto.
   */
  run(command: string, args: string[], timeoutMs: number): Promise<IRunResult>;
}

/** Limite de texto guardado da stderr, para um binário defeituoso não encher a memória. */
const MAX_STDERR_BYTES = 64 * 1024;

/** Executor real, baseado em `child_process.spawn`: um processo por chamada, sem shell. */
export class NodeProcessRunner implements IProcessRunner {
  /** Inicia o processo, mata-o à força no tempo limite e devolve o resultado. */
  run(command: string, args: string[], timeoutMs: number): Promise<IRunResult> {
    return new Promise((resolve, reject) => {
      const child = spawn(command, args, { stdio: ['ignore', 'ignore', 'pipe'], windowsHide: true });
      const chunks: Buffer[] = [];
      let size = 0;
      let timedOut = false;

      child.stderr.on('data', (chunk: Buffer) => {
        if (size < MAX_STDERR_BYTES) {
          chunks.push(chunk);
          size += chunk.length;
        }
      });

      const timer = setTimeout(() => {
        timedOut = true;
        child.kill('SIGKILL');
      }, timeoutMs);

      child.on('error', (error: NodeJS.ErrnoException) => {
        clearTimeout(timer);
        if (error.code === 'ENOENT' || error.code === 'EACCES') {
          reject(new BinaryNotFoundError(`não foi possível executar '${command}': ${error.message}`));
        } else {
          reject(new InternalError(`falha ao iniciar '${command}': ${error.message}`));
        }
      });

      child.on('close', (code, signal) => {
        clearTimeout(timer);
        const stderr = Buffer.concat(chunks).toString('utf8').slice(0, MAX_STDERR_BYTES);
        resolve({ exitCode: code, signal, stderr, timedOut });
      });
    });
  }
}
