const util = require('util');
class Money { constructor(v) { this.v = v; } [util.inspect.custom](depth, options, inspect) { return 'Money<' + this.v + '> depth=' + depth + ' stylize=' + typeof options.stylize + ' ' + (inspect === util.inspect); } }
console.log(new Money(5));
console.log({ wallet: new Money(1) }, [new Money(2)]);
console.log(util.inspect(new Money(3), { depth: 0 }));
class ObjRet { [util.inspect.custom]() { return { replaced: true, n: [1, 2] }; } }
console.log(new ObjRet(), [new ObjRet()]);
class Deep { constructor() { this.inner = { a: { b: { c: 1 } } }; } [util.inspect.custom](d, o, inspect) { return 'Deep ' + inspect(this.inner, { ...o, depth: d }); } }
console.log({ x: { y: new Deep() } });
const viaSymbolFor = { [Symbol.for('nodejs.util.inspect.custom')]() { return 'by-symbol-for'; } };
console.log(viaSymbolFor);
console.log(util.inspect({ toJSON() { return 1; }, toString() { return 'ts'; } }));
class Throwing { [util.inspect.custom]() { return 42; } }
console.log(new Throwing());
