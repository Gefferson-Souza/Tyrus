'use strict';
// Gancho do oráculo (node --require): registra cada módulo CJS carregado e cada leitura
// por fs em jsonl, relativo à raiz do fecho. Só valida o fecho; o produto nunca o usa.
// ESM (import) não é visto: a réplica e o fecho do M1 são CommonJS.
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');

const rootEnv = process.env.TYRUS_CLOSURE_ROOT;
const traceFile = process.env.TYRUS_CLOSURE_TRACE;
if (!rootEnv || !traceFile) {
  throw new Error('closure_trace_hook: defina TYRUS_CLOSURE_ROOT e TYRUS_CLOSURE_TRACE');
}

const root = fs.realpathSync(rootEnv);
const tracePath = path.resolve(traceFile);
const append = fs.appendFileSync;
const seen = new Set();

function classify(file) {
  const relative = path.relative(root, file);
  const outside = relative === '' || relative.startsWith('..') || path.isAbsolute(relative);
  return outside ? { external: true, value: file } : { external: false, value: relative.split(path.sep).join('/') };
}

function record(kind, file) {
  if (typeof file !== 'string' || !path.isAbsolute(file)) return;
  const resolved = path.resolve(file);
  if (resolved === tracePath) return;
  const { external, value } = classify(resolved);
  const effective = external ? 'external' : kind;
  const key = `${effective}\0${value}`;
  if (seen.has(key)) return;
  seen.add(key);
  append(tracePath, `${JSON.stringify({ kind: effective, path: value })}\n`);
}

for (const extension of ['.js', '.json', '.node']) {
  const original = Module._extensions[extension];
  Module._extensions[extension] = function traced(module, filename) {
    record('load', filename);
    return original.call(this, module, filename);
  };
}

const originalReadFileSync = fs.readFileSync;
fs.readFileSync = function tracedRead(target, ...rest) {
  if (typeof target === 'string') record('read', path.resolve(target));
  return originalReadFileSync.call(this, target, ...rest);
};
