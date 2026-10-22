'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
process.on('uncaughtException', (err, origin) => L('uncaught ' + err.message + ' origin=' + origin));
queueMicrotask(() => { throw new Error('from-microtask'); });
setImmediate(() => { throw new Error('from-immediate'); });
process.nextTick(() => { throw new Error('from-tick'); });
Promise.resolve().then(() => L('promise-after'));
setImmediate(() => L('immediate-after'));
