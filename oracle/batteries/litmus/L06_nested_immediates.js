'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
setImmediate(() => L('a'));
setImmediate(() => { L('b'); setImmediate(() => L('b-nested')); });
setImmediate(() => L('c'));
