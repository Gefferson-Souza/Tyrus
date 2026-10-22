'use strict';
const originalFormat = Object.getOwnPropertyDescriptor(Intl.NumberFormat.prototype, 'format').get;
Object.defineProperty(Intl.NumberFormat.prototype, 'format', { configurable: true, get() { const f = originalFormat.call(this); return (v) => f(v) + '​'; } });
