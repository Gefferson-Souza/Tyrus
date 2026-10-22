'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const tp = require('timers/promises');
tp.setImmediate('imm')
  .then((v) => { L('promises-immediate ' + v); return tp.setTimeout(1, 'tm'); })
  .then((v) => L('promises-timeout ' + v));
setImmediate(() => L('callback-immediate'));
process.nextTick(() => L('tick'));
Promise.resolve().then(() => L('promise'));
