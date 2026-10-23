import { dirname, resolve } from 'node:path';

import type { TsApi, TsDiagnostic, TsParsedConfig } from './ts-api.ts';

export interface ProjectConfig {
  readonly configPath: string;
  readonly parsed: TsParsedConfig;
}

export class TsconfigError extends Error {}

export function loadProjectConfig(ts: TsApi, root: string, tsconfigName: string): ProjectConfig {
  const configPath = resolve(root, tsconfigName);
  const read = ts.readConfigFile(configPath, ts.sys.readFile);
  if (read.error !== undefined) {
    throw new TsconfigError(`tsconfig ilegível em ${configPath}: ${formatDiagnostic(ts, read.error)}`);
  }
  const parsed = ts.parseJsonConfigFileContent(
    read.config,
    ts.sys,
    dirname(configPath),
    undefined,
    configPath,
  );
  return { configPath, parsed };
}

export function formatDiagnostic(ts: TsApi, diagnostic: TsDiagnostic): string {
  return `TS${diagnostic.code}: ${ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n')}`;
}
