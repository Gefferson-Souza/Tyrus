import { isAbsolute, relative } from 'node:path';

import { sha256File } from './hash.ts';
import type { EmitResult } from './emit.ts';
import type { TsApi, TsOptions } from './ts-api.ts';

export interface FileFact {
  readonly path: string;
  readonly sha256: string;
}

export interface Facts {
  readonly schema_version: 1;
  readonly typescript: { readonly version: string };
  readonly tsconfig: FileFact;
  readonly emit_options: TsOptions;
  readonly sources: readonly FileFact[];
  readonly outputs: readonly FileFact[];
}

export interface FactsInput {
  readonly ts: TsApi;
  readonly root: string;
  readonly configPath: string;
  readonly outDir: string;
  readonly emit: EmitResult;
}

export function buildFacts(input: FactsInput): Facts {
  const { ts, root, configPath, outDir, emit } = input;
  return {
    schema_version: 1,
    typescript: { version: ts.version },
    tsconfig: { path: portable(root, configPath), sha256: sha256File(configPath) },
    emit_options: portableOptions(root, emit.options),
    sources: inventory(root, emit.sourceFiles),
    outputs: inventory(outDir, emit.writtenFiles),
  };
}

function inventory(base: string, files: readonly string[]): FileFact[] {
  return files
    .map((file) => ({ path: portable(base, file), sha256: sha256File(file) }))
    .sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

function portable(base: string, file: string): string {
  return relative(base, file).split('\\').join('/') || '.';
}

// Caminhos absolutos dependem da máquina; `outDir` é do sidecar e `configFilePath` já está em `tsconfig`.
const MACHINE_BOUND_OPTIONS = new Set(['outDir', 'configFilePath', 'pathsBasePath']);

function portableOptions(root: string, options: TsOptions): TsOptions {
  const entries = Object.entries(options)
    .filter(([name]) => !MACHINE_BOUND_OPTIONS.has(name))
    .map(([name, value]) => [name, relativize(root, value)] as const)
    .sort(([a], [b]) => (a < b ? -1 : 1));
  return Object.fromEntries(entries);
}

function relativize(root: string, value: unknown): unknown {
  if (typeof value === 'string' && isAbsolute(value)) return portable(root, value);
  if (Array.isArray(value)) return value.map((item) => relativize(root, item));
  return value;
}
