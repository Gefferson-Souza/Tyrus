const s = Symbol('desc');
console.log(s, Symbol(), Symbol.for('reg'), Symbol.iterator);
console.log({ [s]: 1, regular: 2, [Symbol.for('k')]: [3] });
console.log([s], new Map([[s, s]]));
console.log(Object(s), Object(Symbol('boxed')));
const t = { [Symbol.toStringTag]: 'Tagged', v: 1 };
console.log(t, Object.prototype.toString.call(t));
console.log({ [Symbol('a b')]: 1, [Symbol('')]: 2 });
