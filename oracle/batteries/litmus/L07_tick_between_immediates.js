'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
setImmediate(() => {
  L('i1');
  process.nextTick(() => L('tick-in-i1'));
  Promise.resolve().then(() => L('promise-in-i1'));
});
setImmediate(() => L('i2'));
