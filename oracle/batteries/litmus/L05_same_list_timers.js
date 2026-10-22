'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
setTimeout(() => L('a-1ms'), 1);
setTimeout(() => L('b-0ms'), 0);
setTimeout(() => L('c-1ms'), 1);
setTimeout(() => L('d-NaN'), NaN);
setTimeout(() => L('e-str'), '1');
setTimeout(() => L('f-neg'), -5);
