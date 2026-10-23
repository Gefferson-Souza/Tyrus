const addOne = (x: number): number => x + 1;
const triple = (x: number): number => x * 3;
function applyTwice(f: (x: number) => number, x: number): number {
    return f(f(x));
}
function compose(f: (x: number) => number, g: (x: number) => number): (x: number) => number {
    return (x: number): number => g(f(x));
}
const composed: (x: number) => number = compose(addOne, triple);
const n: number = 300000000;
let acc: number = 0;
for (let i: number = 0; i < n; i++) {
    acc = acc + composed(i) + applyTwice(addOne, i);
}
console.log(acc);
