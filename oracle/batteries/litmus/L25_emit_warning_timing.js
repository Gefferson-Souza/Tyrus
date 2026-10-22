'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
process.on('warning', (w) => L('warning ' + w.name + ' ' + w.message));
process.emitWarning('custom message', 'CustomWarning');
Promise.resolve().then(() => L('promise'));
process.nextTick(() => L('tick'));
setImmediate(() => L('immediate'));
L('sync');
