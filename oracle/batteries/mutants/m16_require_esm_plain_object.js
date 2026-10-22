'use strict';
const Module = require('module');
const original = Module._load;
Module._load = function patched(request, parent, isMain) {
  const loaded = original.apply(this, arguments);
  if (typeof request === 'string' && request.endsWith('.mjs')) return { ...loaded };
  return loaded;
};
