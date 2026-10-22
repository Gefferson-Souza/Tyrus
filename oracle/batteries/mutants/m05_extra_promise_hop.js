'use strict';
const originalThen = Promise.prototype.then;
let budget = 40;
Promise.prototype.then = function then(onFulfilled, onRejected) {
  if (budget-- <= 0) return originalThen.call(this, onFulfilled, onRejected);
  return originalThen.call(Promise.resolve(), () => originalThen.call(this, onFulfilled, onRejected));
};
