import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { after, describe, test } from 'node:test';

import { validate } from '../src/schema.ts';
import {
  CACHE_DIR, REPLICA_DIR, REPO_ROOT, ensureTypescript, freshDir, matrixVersions, removeDir,
  runSidecar, runTsc, treeDifferences,
} from './support/env.ts';

const schema = JSON.parse(readFileSync(join(REPO_ROOT, 'schemas', 'facts.schema.json'), 'utf8'));

// A réplica instala as dependências reais uma vez: o emit de decorators depende dos tipos resolvidos.
function installedReplica(): string {
  const base = join(CACHE_DIR, 'replica');
  if (!existsSync(join(base, 'node_modules', '@nestjs', 'core', 'package.json'))) {
    mkdirSync(base, { recursive: true });
    for (const name of ['package.json', 'package-lock.json', 'tsconfig.json']) copyFileSync(join(REPLICA_DIR, name), join(base, name));
    execFileSync('npm', ['ci', '--ignore-scripts', '--no-audit', '--no-fund'], { cwd: base, stdio: 'pipe' });
  }
  return base;
}

function projectWithTypescript(version: string): string {
  const root = freshDir(`replica-${version}`);
  const base = installedReplica();
  execFileSync('cp', ['-al', join(base, 'node_modules'), join(root, 'node_modules')]);
  rmSync(join(root, 'node_modules', 'typescript'), { recursive: true, force: true });
  cpSync(ensureTypescript(version), join(root, 'node_modules', 'typescript'), { recursive: true });
  copyFileSync(join(REPLICA_DIR, 'tsconfig.json'), join(root, 'tsconfig.json'));
  cpSync(join(REPLICA_DIR, 'src'), join(root, 'src'), { recursive: true });
  return root;
}

for (const version of matrixVersions()) {
  describe(`réplica com TypeScript ${version}`, () => {
    const root = projectWithTypescript(version);
    after(() => removeDir(root));

    test('emit igual byte a byte ao de tsc -p da mesma versão', () => {
      const sidecar = runSidecar(root, join(root, 'sidecar'));
      const control = runTsc(join(root, 'node_modules', 'typescript'), root, join(root, 'tsc', 'emit'));
      assert.equal(sidecar.status === 0, control.status === 0, sidecar.stderr);
      assert.deepEqual(treeDifferences(join(root, 'sidecar', 'emit'), join(root, 'tsc', 'emit')), []);
      assert.ok(existsSync(join(root, 'sidecar', 'emit', 'main.js')));
    });

    test('facts.json da réplica valida no esquema', () => {
      const facts = JSON.parse(readFileSync(join(root, 'sidecar', 'facts.json'), 'utf8'));
      assert.deepEqual(validate(schema, facts), []);
      assert.equal(facts.typescript.version, version);
    });
  });
}
