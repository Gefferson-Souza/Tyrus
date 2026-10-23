import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { after, describe, test } from 'node:test';

import {
  ensureTypescript, freshDir, linkTypescript, listFiles, removeDir, run, CLI,
  runSidecar, writeFixtureProject,
} from './support/env.ts';

const VERSION = '5.9.3';

// Hash independente de src/hash.ts: um teste que reusa a função sob teste não detecta defeito nela.
const sha256File = (path: string): string => createHash('sha256').update(readFileSync(path)).digest('hex');

interface FileFact { path: string; sha256: string }
const readFacts = (out: string) => JSON.parse(readFileSync(join(out, 'facts.json'), 'utf8'));

describe('conteúdo do facts.json', () => {
  const root = freshDir('facts');
  after(() => removeDir(root));
  writeFixtureProject(root);
  linkTypescript(root, ensureTypescript(VERSION));
  const out = join(root, 'sidecar');
  const result = runSidecar(root, out);
  const facts = readFacts(out);

  test('o sidecar sai 0 no projeto válido', () => assert.equal(result.status, 0, result.stderr));

  test('outputs lista exatamente os arquivos emitidos, com o sha256 real', () => {
    const emitDir = join(out, 'emit');
    const paths = facts.outputs.map((file: FileFact) => file.path);
    assert.deepEqual(paths, listFiles(emitDir));
    assert.notEqual(paths.length, 0);
    for (const file of facts.outputs as FileFact[]) {
      assert.equal(file.sha256, sha256File(join(emitDir, file.path)), file.path);
    }
  });

  test('sources traz as fontes relativas à raiz com o sha256 real', () => {
    for (const file of facts.sources as FileFact[]) {
      assert.equal(file.sha256, sha256File(join(root, file.path)), file.path);
    }
    assert.equal(facts.sources.length, 2);
  });

  test('tsconfig.sha256 é o hash do arquivo', () => {
    assert.equal(facts.tsconfig.path, 'tsconfig.json');
    assert.equal(facts.tsconfig.sha256, sha256File(join(root, 'tsconfig.json')));
  });
});

describe('política de saída', () => {
  test('erro de tipo: sai 1, mas emite e escreve o facts.json', () => {
    const root = freshDir('type-error');
    after(() => removeDir(root));
    writeFixtureProject(root);
    writeFileSync(join(root, 'src', 'broken.ts'), 'export const n: number = "texto";\n');
    linkTypescript(root, ensureTypescript(VERSION));
    const out = join(root, 'sidecar');
    const result = runSidecar(root, out);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /TS2322/);
    assert.ok(listFiles(join(out, 'emit')).includes('broken.js'));
    assert.ok(readFacts(out).outputs.some((file: FileFact) => file.path === 'broken.js'));
  });

  test('tsconfig inexistente: sai 2 sem escrever facts.json', () => {
    const root = freshDir('no-tsconfig');
    after(() => removeDir(root));
    linkTypescript(root, ensureTypescript(VERSION));
    const result = run(process.execPath, [CLI, '--project', root, '--out', join(root, 'sidecar'), '--tsconfig', 'nao-existe.json']);
    assert.equal(result.status, 2);
    assert.notEqual(result.stderr, '');
    assert.throws(() => readFileSync(join(root, 'sidecar', 'facts.json')));
  });
});

describe('o sidecar só escreve dentro de --out', () => {
  test('declarationDir e tsBuildInfoFile do projeto são reancorados sob o outDir', () => {
    const root = freshDir('decl');
    after(() => removeDir(root));
    writeFixtureProject(root);
    mkdirSync(join(root, 'types'), { recursive: true });
    writeFileSync(join(root, 'tsconfig.json'), JSON.stringify({
      compilerOptions: {
        module: 'commonjs', target: 'ES2021', lib: ['ES2021'], outDir: './dist', rootDir: './src',
        declaration: true, declarationDir: './types', incremental: true, tsBuildInfoFile: './info/app.tsbuildinfo',
        strict: true, skipLibCheck: true, types: [], experimentalDecorators: true,
      },
      include: ['src/**/*.ts'],
    }));
    linkTypescript(root, ensureTypescript(VERSION));
    const before = listFiles(root);
    const out = join(root, 'sidecar');
    const result = runSidecar(root, out);
    assert.equal(result.status, 0, result.stderr);
    const outputs = readFacts(out).outputs.map((file: FileFact) => file.path);
    assert.equal(outputs.some((path: string) => path.startsWith('..')), false, outputs.join(','));
    assert.ok(outputs.some((path: string) => path.endsWith('.d.ts')));
    const created = listFiles(root).filter((file) => !before.includes(file));
    assert.deepEqual(created.filter((file) => !file.startsWith('sidecar/')), []);
  });
});

describe('portabilidade do facts.json', () => {
  test('emit_options não depende de onde --out fica', () => {
    const root = freshDir('portable');
    after(() => removeDir(root));
    writeFixtureProject(root);
    writeFileSync(join(root, 'tsconfig.json'), JSON.stringify({
      compilerOptions: { module: 'commonjs', outDir: './dist', rootDir: './src', declaration: true, declarationDir: './types', types: [] },
      include: ['src/**/*.ts'],
    }));
    linkTypescript(root, ensureTypescript(VERSION));
    runSidecar(root, join(root, 'a'));
    runSidecar(root, join(root, 'deep', 'b'));
    assert.deepEqual(readFacts(join(root, 'a')), readFacts(join(root, 'deep', 'b')));
  });
});
