import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { after, describe, test } from 'node:test';

import { validate } from '../src/schema.ts';
import {
  REPO_ROOT, ensureTypescript, freshDir, linkTypescript, matrixVersions, removeDir,
  runSidecar, runTsc, treeDifferences, writeFixtureProject,
} from './support/env.ts';

const schema = JSON.parse(readFileSync(join(REPO_ROOT, 'schemas', 'facts.schema.json'), 'utf8'));

for (const version of matrixVersions()) {
  describe(`fixture com TypeScript ${version}`, () => {
    const root = freshDir(`fixture-${version}`);
    after(() => removeDir(root));
    const typescriptDir = ensureTypescript(version);
    writeFixtureProject(root);
    linkTypescript(root, typescriptDir);
    const sidecarOut = join(root, 'sidecar');
    // mesma profundidade de outDir: o sourceMap embute o caminho relativo até as fontes
    const tscOut = join(root, 'tsc', 'emit');

    test('emit igual byte a byte ao de tsc -p da mesma versão', () => {
      const sidecar = runSidecar(root, sidecarOut);
      const control = runTsc(typescriptDir, root, tscOut);
      assert.equal(sidecar.status, control.status, sidecar.stderr);
      assert.deepEqual(treeDifferences(join(sidecarOut, 'emit'), tscOut), []);
    });

    test('facts.json valida no esquema e registra a versão usada', () => {
      const facts = JSON.parse(readFileSync(join(sidecarOut, 'facts.json'), 'utf8'));
      assert.deepEqual(validate(schema, facts), []);
      assert.equal(facts.typescript.version, version);
      assert.deepEqual(facts.sources.map((file: { path: string }) => file.path), ['src/main.ts', 'src/nested/dep.ts']);
    });

    test('as opções do projeto passam intactas, só o outDir muda', () => {
      const facts = JSON.parse(readFileSync(join(sidecarOut, 'facts.json'), 'utf8'));
      assert.equal(facts.emit_options.experimentalDecorators, true);
      assert.equal(facts.emit_options.rootDir, 'src');
      assert.equal('outDir' in facts.emit_options, false);
    });
  });
}
