'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
setTimeout(() => {
  L('t1');
  process.nextTick(() => L('tick-in-t1'));
  Promise.resolve().then(() => L('promise-in-t1'));
}, 1);
setTimeout(() => L('t2'), 1);
