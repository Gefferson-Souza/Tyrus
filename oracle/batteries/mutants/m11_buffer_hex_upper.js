'use strict';
const original = Buffer.prototype.toString;
Buffer.prototype.toString = function toString(encoding, ...rest) {
  const text = original.call(this, encoding, ...rest);
  return encoding === 'hex' ? text.toUpperCase() : text;
};
