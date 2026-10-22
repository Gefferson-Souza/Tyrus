'use strict';
const original = Date.prototype.getTimezoneOffset;
Date.prototype.getTimezoneOffset = function getTimezoneOffset() { return original.call(this) + 1; };
