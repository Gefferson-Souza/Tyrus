'use strict';
const originalSort = Array.prototype.sort;
Array.prototype.sort = function sort(compare) {
  if (typeof compare !== 'function') return originalSort.call(this);
  return originalSort.call(this, (a, b) => -compare(b, a));
};
