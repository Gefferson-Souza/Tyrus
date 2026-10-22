'use strict';
const original = String.prototype.normalize;
String.prototype.normalize = function normalize(form) { return form === 'NFKD' ? String(this) : original.call(this, form); };
