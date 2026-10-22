'use strict';
global.queueMicrotask = (fn) => setImmediate(fn);
