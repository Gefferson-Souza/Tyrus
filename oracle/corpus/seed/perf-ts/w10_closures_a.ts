const addOne = (x: number): number => x + 1;
const triple = (x: number): number => x * 3;
const n: number = 300000000;
let acc: number = 0;
for (let i: number = 0; i < n; i++) {
    acc = acc + triple(addOne(i)) + addOne(addOne(i));
}
console.log(acc);
