'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
let rounds = 0;
process.on('beforeExit', (code) => {
  L('beforeExit ' + code + ' round=' + rounds);
  if (rounds === 0) { rounds += 1; setImmediate(() => L('scheduled-in-beforeExit')); }
});
process.on('exit', (code) => {
  L('exit-1 ' + code);
  process.nextTick(() => L('tick-in-exit-never'));
  setImmediate(() => L('immediate-in-exit-never'));
});
process.on('exit', () => L('exit-2'));
process.exitCode = 3;
setTimeout(() => L('timer'), 1);
