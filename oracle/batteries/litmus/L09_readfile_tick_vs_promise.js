'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
fs.readFile(__filename, () => {
  Promise.resolve().then(() => L('promise'));
  process.nextTick(() => L('tick'));
  L('callback');
});
