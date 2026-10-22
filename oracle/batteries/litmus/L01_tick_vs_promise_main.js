'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
Promise.resolve().then(() => L('promise'));
process.nextTick(() => L('tick'));
queueMicrotask(() => L('microtask'));
L('sync');
