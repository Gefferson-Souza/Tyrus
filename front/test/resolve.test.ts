import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { after, test } from 'node:test';

import { MISSING_TYPESCRIPT_CODE } from '../src/resolve.ts';
import { REPO_ROOT, freshDir, removeDir, runSidecar } from './support/env.ts';

const root = freshDir('resolve');
after(() => removeDir(root));

test('projeto sem typescript local sai 2 com TYRUS-E7001', () => {
  writeFileSync(join(root, 'tsconfig.json'), '{}');
  const result = runSidecar(root, join(root, 'out'));
  assert.equal(result.status, 2);
  assert.match(result.stderr, /^TYRUS-E7001: /);
});

test('o código E7001 existe no registro de erros do Rust', () => {
  const registry = readFileSync(join(REPO_ROOT, 'crates', 'tyrus_diagnostics', 'src', 'lib.rs'), 'utf8');
  assert.ok(registry.includes(`"${MISSING_TYPESCRIPT_CODE}"`));
});
