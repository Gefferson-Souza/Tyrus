import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { parseArgs } from 'node:util';

import { formatDiagnostic, loadProjectConfig, TsconfigError } from './config.ts';
import { emitProject, type EmitResult } from './emit.ts';
import { buildFacts } from './facts.ts';
import { MissingTypescriptError, resolveProjectTypescript } from './resolve.ts';
import { validate } from './schema.ts';
import type { TsApi } from './ts-api.ts';

const EXIT_OK = 0;
const EXIT_DIAGNOSTICS = 1;
const EXIT_REFUSED = 2;
const FACTS_SCHEMA = new URL('../../schemas/facts.schema.json', import.meta.url);

function run(argv: string[]): number {
  const { values } = parseArgs({
    args: argv,
    options: {
      project: { type: 'string' },
      out: { type: 'string' },
      tsconfig: { type: 'string', default: 'tsconfig.json' },
    },
  });
  if (values.project === undefined || values.out === undefined) {
    process.stderr.write('uso: cli.ts --project <raiz> --out <diretório> [--tsconfig <arquivo>]\n');
    return EXIT_REFUSED;
  }
  const root = resolve(values.project);
  const out = resolve(values.out);
  const ts = resolveProjectTypescript(root);
  const config = loadProjectConfig(ts, root, values.tsconfig ?? 'tsconfig.json');
  const outDir = join(out, 'emit');
  mkdirSync(outDir, { recursive: true });
  const emit = emitProject(ts, config, outDir);
  const facts = buildFacts({ ts, root, configPath: config.configPath, outDir, emit });
  const problems = validate(JSON.parse(readFileSync(FACTS_SCHEMA, 'utf8')), facts);
  if (problems.length > 0) throw new Error(`facts.json fora do esquema: ${problems.join('; ')}`);
  writeFileSync(join(out, 'facts.json'), `${JSON.stringify(facts, null, 2)}\n`);
  return reportDiagnostics(ts, emit.diagnostics);
}

function reportDiagnostics(ts: TsApi, diagnostics: EmitResult['diagnostics']): number {
  const errors = diagnostics.filter((d) => d.category === ts.DiagnosticCategory.Error);
  for (const diagnostic of errors) process.stderr.write(`${formatDiagnostic(ts, diagnostic)}\n`);
  return errors.length > 0 ? EXIT_DIAGNOSTICS : EXIT_OK;
}

try {
  process.exitCode = run(process.argv.slice(2));
} catch (error) {
  if (error instanceof MissingTypescriptError || error instanceof TsconfigError) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = EXIT_REFUSED;
  } else {
    throw error;
  }
}
