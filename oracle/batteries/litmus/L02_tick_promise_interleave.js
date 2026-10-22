'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
process.nextTick(() => {
  L('tick1');
  Promise.resolve().then(() => L('promise-in-tick1'));
  process.nextTick(() => L('tick-in-tick1'));
});
Promise.resolve().then(() => {
  L('promise1');
  process.nextTick(() => L('tick-in-promise1'));
  Promise.resolve().then(() => L('promise-in-promise1'));
});
process.nextTick(() => L('tick2'));
Promise.resolve().then(() => L('promise2'));
L('sync');
