interface Inner { c: number; }
interface Outer { b?: Inner; }
const o1: Outer = {};
const o2: Outer = { b: { c: 5 } };
console.log(o1.b?.c ?? -1);
console.log(o2.b?.c ?? -1);
