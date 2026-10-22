'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const { Writable } = require('stream');
const w = new Writable({
  write(chunk, enc, cb) { L('write ' + chunk); cb(); },
  final(cb) { L('final'); cb(); },
});
w.on('finish', () => L('finish'));
w.on('close', () => L('close'));
w.write('a', () => L('cb-a'));
w.write('b', () => L('cb-b'));
w.end('c', () => L('end-cb'));
process.nextTick(() => L('tick'));
Promise.resolve().then(() => L('promise'));
L('sync');
