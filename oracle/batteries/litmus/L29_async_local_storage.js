'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const { AsyncLocalStorage } = require('async_hooks');
const { EventEmitter } = require('events');
const als = new AsyncLocalStorage();
const e = new EventEmitter();
const show = (where) => L(where + ' ' + JSON.stringify(als.getStore() ?? null));
als.run({ id: 'ctx' }, () => {
  e.on('x', () => show('emitter-listener'));
  show('sync');
  Promise.resolve().then(() => show('promise'));
  process.nextTick(() => show('tick'));
  queueMicrotask(() => show('microtask'));
  (async () => { await null; show('await'); })();
  setImmediate(() => {
    show('immediate');
    fs.readFile(__filename, () => {
      show('readFile');
      setTimeout(() => { show('timeout'); e.emit('x'); }, 1);
    });
  });
});
setImmediate(() => show('outside'));
