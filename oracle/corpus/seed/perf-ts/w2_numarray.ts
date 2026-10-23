function build(n: number): number[] {
    const arr: number[] = [];
    for (let i: number = 0; i < n; i++) {
        arr.push(i * 2 + 1);
    }
    return arr;
}
function pipeline(data: number[]): number {
    return data
        .map((x: number): number => x * 3)
        .filter((x: number): boolean => x % 4 === 3)
        .reduce((acc: number, x: number): number => acc + x, 0);
}
const n: number = 1000000;
const rounds: number = 10;
const data: number[] = build(n);
let total: number = 0;
for (let r: number = 0; r < rounds; r++) {
    total = (total + pipeline(data)) % 1000000007;
}
console.log(total);
