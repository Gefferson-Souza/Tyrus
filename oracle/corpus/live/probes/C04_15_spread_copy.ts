const a: number[] = [1, 2, 3];
let b: number[] = [...a];
b.push(9);
console.log(a.length);
console.log(b.length);
