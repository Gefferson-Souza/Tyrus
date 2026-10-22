'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
fs.readFile(__filename, () => {
  setTimeout(() => L('timeout0'), 0);
  setImmediate(() => L('immediate'));
  process.nextTick(() => L('tick'));
  Promise.resolve().then(() => L('promise'));
  L('io-callback');
});
