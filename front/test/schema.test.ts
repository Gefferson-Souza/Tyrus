import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { validate } from '../src/schema.ts';
import { REPO_ROOT } from './support/env.ts';

const schema = JSON.parse(readFileSync(join(REPO_ROOT, 'schemas', 'facts.schema.json'), 'utf8'));
const HASH = 'a'.repeat(64);
const valid = () => ({
  schema_version: 1,
  typescript: { version: '5.1.6' },
  tsconfig: { path: 'tsconfig.json', sha256: HASH },
  emit_options: { target: 7 },
  sources: [{ path: 'src/main.ts', sha256: HASH }],
  outputs: [{ path: 'main.js', sha256: HASH }],
});

test('aceita um facts.json válido', () => {
  assert.deepEqual(validate(schema, valid()), []);
});

test('rejeita campo extra na raiz', () => {
  assert.notDeepEqual(validate(schema, { ...valid(), extra: true }), []);
});

test('rejeita hash malformado', () => {
  const facts = { ...valid(), sources: [{ path: 'a.ts', sha256: 'xyz' }] };
  assert.notDeepEqual(validate(schema, facts), []);
});

test('rejeita versão de esquema desconhecida', () => {
  assert.notDeepEqual(validate(schema, { ...valid(), schema_version: 2 }), []);
});

test('rejeita campo obrigatório ausente', () => {
  const { outputs: _removed, ...incomplete } = valid();
  assert.notDeepEqual(validate(schema, incomplete), []);
});

test('palavra-chave de esquema sem suporte lança erro', () => {
  assert.throws(() => validate({ type: 'object', oneOf: [] }, {}), /sem suporte/);
});
