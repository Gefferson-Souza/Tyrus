import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync, symlinkSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

export const FRONT_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
export const REPO_ROOT = join(FRONT_DIR, '..');
export const CLI = join(FRONT_DIR, 'src', 'cli.ts');
export const REPLICA_DIR = join(REPO_ROOT, 'oracle', 'apps', 'replica');

// Fora de /tmp: a matriz instala 3 TypeScript e a réplica (~180 MB) e é reaproveitada entre execuções.
export const CACHE_DIR = process.env.TYRUS_TS_MATRIX_DIR ?? join(homedir(), '.local', 'share', 'tyrus-ts-matrix');

export function matrixVersions(): string[] {
  const file = join(FRONT_DIR, 'test', 'ts-matrix.json');
  return (JSON.parse(readFileSync(file, 'utf8')) as { versions: string[] }).versions;
}

export function ensureTypescript(version: string): string {
  const prefix = join(CACHE_DIR, `ts-${version}`);
  const pkg = join(prefix, 'node_modules', 'typescript');
  if (!existsSync(join(pkg, 'package.json'))) {
    mkdirSync(prefix, { recursive: true });
    execFileSync('npm', ['install', '--prefix', prefix, '--no-save', '--ignore-scripts', '--no-audit', '--no-fund', `typescript@${version}`], { stdio: 'pipe' });
  }
  return pkg;
}

export function freshDir(label: string): string {
  mkdirSync(join(CACHE_DIR, 'work'), { recursive: true });
  return mkdtempSync(join(CACHE_DIR, 'work', `${label}-`));
}

export function removeDir(path: string): void {
  rmSync(path, { recursive: true, force: true });
}

export function linkTypescript(projectRoot: string, typescriptDir: string): void {
  mkdirSync(join(projectRoot, 'node_modules'), { recursive: true });
  symlinkSync(typescriptDir, join(projectRoot, 'node_modules', 'typescript'), 'dir');
}

export interface RunResult {
  readonly status: number;
  readonly stderr: string;
}

export function run(command: string, args: string[]): RunResult {
  try {
    execFileSync(command, args, { stdio: 'pipe', maxBuffer: 64 * 1024 * 1024 });
    return { status: 0, stderr: '' };
  } catch (error) {
    const failure = error as { status?: number; stderr?: Buffer };
    return { status: failure.status ?? -1, stderr: String(failure.stderr ?? '') };
  }
}

export function runSidecar(projectRoot: string, outDir: string): RunResult {
  return run(process.execPath, [CLI, '--project', projectRoot, '--out', outDir]);
}

export function runTsc(typescriptDir: string, projectRoot: string, outDir: string): RunResult {
  return run(process.execPath, [join(typescriptDir, 'bin', 'tsc'), '-p', projectRoot, '--outDir', outDir]);
}

export function listFiles(dir: string): string[] {
  return readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => relative(dir, join(entry.parentPath, entry.name)))
    .sort();
}

export function treeDifferences(left: string, right: string): string[] {
  const a = listFiles(left);
  const b = listFiles(right);
  const names = [...new Set([...a, ...b])].sort();
  return names.filter((name) => {
    const inBoth = a.includes(name) && b.includes(name);
    return !inBoth || !readFileSync(join(left, name)).equals(readFileSync(join(right, name)));
  });
}

export function writeFixtureProject(root: string): void {
  mkdirSync(join(root, 'src', 'nested'), { recursive: true });
  writeFileSync(join(root, 'tsconfig.json'), JSON.stringify({
    compilerOptions: {
      module: 'commonjs', target: 'ES2021', lib: ['ES2021'], outDir: './dist', rootDir: './src',
      experimentalDecorators: true, emitDecoratorMetadata: true, esModuleInterop: true,
      sourceMap: true, removeComments: false, strict: true, skipLibCheck: true, types: [],
    },
    include: ['src/**/*.ts'],
  }, null, 2));
  writeFileSync(join(root, 'src', 'nested', 'dep.ts'), 'export class Dep { readonly id = 1; }\n');
  writeFileSync(join(root, 'src', 'main.ts'), [
    "import { Dep } from './nested/dep';",
    'function mark(target: object, key: string): void { void target; void key; }',
    'export class Service {',
    '  @mark',
    '  dep: Dep = new Dep();',
    '  run(label: string): string { return `${label}:${this.dep.id}`; }',
    '}',
    '',
  ].join('\n'));
}

export function isDirectory(path: string): boolean {
  return existsSync(path) && statSync(path).isDirectory();
}
