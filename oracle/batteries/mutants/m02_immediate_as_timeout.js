'use strict';
const original = global.setImmediate;
global.setImmediate = (fn, ...args) => setTimeout(fn, 0, ...args);
void original;
