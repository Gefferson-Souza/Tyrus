'use strict';
const util = require('util');
const originalInspect = util.inspect;
const patched = function inspect(value, ...rest) { return originalInspect.call(this, value, ...rest).replace(/'/g, '"'); };
Object.assign(patched, originalInspect);
util.inspect = patched;
const originalLog = console.log;
console.log = (...args) => originalLog(...args.map((a) => (typeof a === 'string' ? a : originalInspect(a).replace(/'/g, '"'))));
