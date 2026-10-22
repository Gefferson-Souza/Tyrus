'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
async function f() { L('f1'); await null; L('f2'); await null; L('f3'); }
f();
Promise.resolve().then(() => L('p1')).then(() => L('p2')).then(() => L('p3'));
L('sync');
