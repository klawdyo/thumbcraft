import { spawn } from 'node:child_process';

import { BinaryNotFoundError, InternalError } from './errors.ts';

/** Resultado de uma execução do binário. */
export interface IRunResult {
  /** Código de saída, ou `null` quando o processo morreu por sinal. */
  exitCode: number | null;
  /** Sinal que encerrou o processo, quando houve. */
  signal: string | null;
  /** Texto da stdout (limitado a alguns KB; o binário só escreve texto curto nela com `--detect`). */
  stdout: string;
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
   * @param stdin Bytes enviados à stdin do processo, que depois é fechada; sem isso a stdin é ignorada.
   */
  run(command: string, args: string[], timeoutMs: number, stdin?: Buffer): Promise<IRunResult>;
}

/** Limite de texto guardado de cada fluxo, para um binário defeituoso não encher a memória. */
const MAX_CAPTURED_BYTES = 64 * 1024;

/** Acumula o texto de um fluxo até o limite e descarta o resto. */
class BoundedText {
  private readonly chunks: Buffer[] = [];
  private size = 0;

  add(chunk: Buffer): void {
    if (this.size < MAX_CAPTURED_BYTES) {
      this.chunks.push(chunk);
      this.size += chunk.length;
    }
  }

  text(): string {
    return Buffer.concat(this.chunks).toString('utf8').slice(0, MAX_CAPTURED_BYTES);
  }
}

/** Executor real, baseado em `child_process.spawn`: um processo por chamada, sem shell. */
export class NodeProcessRunner implements IProcessRunner {
  /** Inicia o processo, mata-o à força no tempo limite e devolve o resultado. */
  run(command: string, args: string[], timeoutMs: number, stdin?: Buffer): Promise<IRunResult> {
    return new Promise((resolve, reject) => {
      const child = spawn(command, args, { stdio: [stdin === undefined ? 'ignore' : 'pipe', 'pipe', 'pipe'], windowsHide: true });
      const stdout = new BoundedText();
      const stderr = new BoundedText();
      let timedOut = false;

      child.stdout?.on('data', (chunk: Buffer) => stdout.add(chunk));
      child.stderr?.on('data', (chunk: Buffer) => stderr.add(chunk));
      if (stdin !== undefined && child.stdin !== null) {
        // O binário pode terminar sem ler tudo (erro de uso): o EPIPE não é falha de quem chama.
        child.stdin.on('error', () => {});
        child.stdin.end(stdin);
      }

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
        resolve({ exitCode: code, signal, stdout: stdout.text(), stderr: stderr.text(), timedOut });
      });
    });
  }
}
