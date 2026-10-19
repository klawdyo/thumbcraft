import type { IProcessRunner, IRunResult } from '../process-runner.ts';

/** Chamada registrada pelo executor falso. */
export interface IRecordedCall {
  /** Caminho do executável. */
  command: string;
  /** Argumentos passados ao executável. */
  args: string[];
  /** Tempo limite pedido, em milissegundos. */
  timeoutMs: number;
}

/** Resultado de sucesso usado como padrão pelo executor falso. */
export const OK_RESULT: IRunResult = { exitCode: 0, signal: null, stderr: '', timedOut: false };

/**
 * Executor de processos falso: registra as chamadas e devolve um resultado programado,
 * sem iniciar processo nenhum. Um gancho opcional simula os efeitos colaterais do binário.
 */
export class FakeProcessRunner implements IProcessRunner {
  /** Chamadas feitas, na ordem. */
  readonly calls: IRecordedCall[] = [];

  private readonly result: IRunResult;
  private readonly onRun: (call: IRecordedCall) => Promise<void> | void;

  /**
   * @param result Resultado devolvido a cada chamada.
   * @param onRun Gancho executado a cada chamada (por exemplo, para gravar o arquivo de saída).
   */
  constructor(result: IRunResult = OK_RESULT, onRun: (call: IRecordedCall) => Promise<void> | void = () => {}) {
    this.result = result;
    this.onRun = onRun;
  }

  /** Registra a chamada, executa o gancho e devolve o resultado programado. */
  async run(command: string, args: string[], timeoutMs: number): Promise<IRunResult> {
    const call = { command, args, timeoutMs };
    this.calls.push(call);
    await this.onRun(call);
    return this.result;
  }
}
