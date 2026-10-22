'use strict';
const originalSin = Math.sin;
Math.sin = (x) => originalSin(x) * (1 + 2 ** -52);
