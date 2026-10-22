'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
queueMicrotask(() => L('qm1'));
Promise.resolve().then(() => L('p1')).then(() => L('p2'));
queueMicrotask(() => { L('qm2'); queueMicrotask(() => L('qm-in-qm2')); });
Promise.resolve().then(() => L('p3'));
L('sync');
