class A { constructor() { this.x = 1; } get y() { return 2; } static s = 3; }
class B extends A { #priv = 1; z = [1]; }
console.log(new A(), new B());
console.log(A, B, class {}, class extends A {});
console.log(Object.create(null), Object.assign(Object.create(null), { a: 1 }));
console.log(Object.create({ inherited: 1 }));
class Tagged { get [Symbol.toStringTag]() { return 'Custom'; } }
console.log(new Tagged(), Object.create(A.prototype));
console.log(function f() {}, () => {}, async function g() {}, function* h() {}, async function* i() {}, async () => {});
function withProps() {}
withProps.extra = 1;
console.log(withProps, Math.max, class Z { static a = 1; });
class MyArr extends Array {}
class MyMap extends Map {}
console.log(MyArr.from([1, 2]), new MyMap([[1, 2]]));
console.log(Object.setPrototypeOf({ a: 1 }, null), new (class extends Error {})('m').name);
