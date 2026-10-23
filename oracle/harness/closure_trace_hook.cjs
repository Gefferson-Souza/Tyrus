'use strict';
// Gancho do oráculo (node --require): registra cada módulo CJS carregado e cada leitura
// por fs em jsonl, relativo à raiz do fecho. Só valida o fecho; o produto nunca o usa.
// Limites declarados: ESM (import) não é visto (a réplica e o fecho do M1 são CommonJS) e o
// package.json consultado na resolução é inferido pela raiz do pacote do arquivo resolvido,
// pois o resolvedor do Node o lê por binding interno, fora de fs.
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

function targetPath(target) {
  return typeof target === 'string' ? path.resolve(target) : null;
}

// Registra só depois da leitura bem-sucedida: sonda de arquivo inexistente não é dependência.
const originalReadFileSync = fs.readFileSync;
fs.readFileSync = function tracedRead(target, ...rest) {
  const result = originalReadFileSync.call(this, target, ...rest);
  const file = targetPath(target);
  if (file) record('read', file);
  return result;
};

const originalReadFile = fs.readFile;
fs.readFile = function tracedReadFile(target, ...rest) {
  const callback = rest.pop();
  const file = targetPath(target);
  return originalReadFile.call(this, target, ...rest, function tracedCallback(error, ...values) {
    if (!error && file) record('read', file);
    return callback.call(this, error, ...values);
  });
};

const originalPromiseReadFile = fs.promises.readFile;
fs.promises.readFile = async function tracedPromiseRead(target, ...rest) {
  const result = await originalPromiseReadFile.call(this, target, ...rest);
  const file = targetPath(target);
  if (file) record('read', file);
  return result;
};

const originalCreateReadStream = fs.createReadStream;
fs.createReadStream = function tracedStream(target, ...rest) {
  const stream = originalCreateReadStream.call(this, target, ...rest);
  const file = targetPath(target);
  if (file) stream.once('open', () => record('read', file));
  return stream;
};

function packageRoot(file) {
  const parts = path.relative(root, file).split(path.sep);
  const last = parts.lastIndexOf('node_modules');
  if (last < 0 || parts.length <= last + 1) return null;
  const depth = parts[last + 1].startsWith('@') ? last + 3 : last + 2;
  return path.join(root, ...parts.slice(0, depth));
}

const originalFindPath = Module._findPath;
Module._findPath = function tracedFindPath(...args) {
  const found = originalFindPath.apply(this, args);
  if (typeof found === 'string') {
    const dir = packageRoot(found);
    const manifest = dir && path.join(dir, 'package.json');
    if (manifest && fs.existsSync(manifest)) record('read', manifest);
  }
  return found;
};
