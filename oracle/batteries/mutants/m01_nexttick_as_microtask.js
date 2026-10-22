'use strict';
process.nextTick = (fn, ...args) => queueMicrotask(() => fn(...args));
