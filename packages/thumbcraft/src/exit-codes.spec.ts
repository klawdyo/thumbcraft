import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  InternalError,
  InvalidFileError,
  ProcessCrashError,
  ThumbcraftError,
  TimeoutError,
  UnsupportedFormatError,
  UsageError,
} from './errors.ts';
import { errorFromRun } from './exit-codes.ts';
import type { IRunResult } from './process-runner.ts';

const run = (partial: Partial<IRunResult>): IRunResult => ({ exitCode: 0, signal: null, stderr: '', timedOut: false, ...partial });

describe('errorFromRun', () => {
  it('returns null on exit code 0', () => {
    assert.equal(errorFromRun(run({}), 30_000), null);
  });

  it('maps exit codes 2, 3, 4 and 5 to typed errors', () => {
    assert.ok(errorFromRun(run({ exitCode: 2 }), 1) instanceof UsageError);
    assert.ok(errorFromRun(run({ exitCode: 3 }), 1) instanceof UnsupportedFormatError);
    assert.ok(errorFromRun(run({ exitCode: 4 }), 1) instanceof InvalidFileError);
    assert.ok(errorFromRun(run({ exitCode: 5 }), 1) instanceof InternalError);
  });

  it('keeps the exit code and strips the thumbcraft prefix from stderr', () => {
    const error = errorFromRun(run({ exitCode: 4, stderr: 'thumbcraft: o arquivo não tem página 9: só há 3 no total\n' }), 1);
    assert.ok(error instanceof InvalidFileError);
    assert.equal(error.exitCode, 4);
    assert.equal(error.message, 'o arquivo não tem página 9: só há 3 no total');
    assert.equal(error.stderr, 'thumbcraft: o arquivo não tem página 9: só há 3 no total\n');
  });

  it('uses a default message when stderr is empty', () => {
    const error = errorFromRun(run({ exitCode: 3 }), 1);
    assert.ok(error instanceof ThumbcraftError);
    assert.match(error.message, /formato/i);
  });

  it('maps abort statuses (134) and signals to ProcessCrashError', () => {
    const abort = errorFromRun(run({ exitCode: 134 }), 1);
    assert.ok(abort instanceof ProcessCrashError);
    assert.equal(abort.exitCode, 134);
    const signal = errorFromRun(run({ exitCode: null, signal: 'SIGABRT' }), 1);
    assert.ok(signal instanceof ProcessCrashError);
    assert.match(signal.message, /SIGABRT/);
  });

  it('maps any unexpected status, such as 1 or 124, to ProcessCrashError', () => {
    assert.ok(errorFromRun(run({ exitCode: 1 }), 1) instanceof ProcessCrashError);
    assert.ok(errorFromRun(run({ exitCode: 124 }), 1) instanceof ProcessCrashError);
  });

  it('maps a timeout to TimeoutError, whatever the exit code was', () => {
    const error = errorFromRun(run({ exitCode: null, signal: 'SIGKILL', timedOut: true }), 1500);
    assert.ok(error instanceof TimeoutError);
    assert.match(error.message, /1500/);
  });

  it('keeps every typed error as a ThumbcraftError', () => {
    for (const code of [2, 3, 4, 5, 134]) {
      assert.ok(errorFromRun(run({ exitCode: code }), 1) instanceof ThumbcraftError);
    }
  });
});
