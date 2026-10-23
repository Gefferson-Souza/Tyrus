function generate(n: number): number[] {
    let out: number[] = [];
    let seed: number = 12345;
    for (let i: number = 0; i < n; i++) {
        seed = (seed * 1664525 + 1013904223) % 4294967296;
        out.push(seed);
    }
    return out;
}
function checksum(sorted: number[]): number {
    let check: number = 0;
    for (const v of sorted) {
        check = (check * 31 + v) % 1000000007;
    }
    return check;
}
const n: number = 1000000;
let arr: number[] = generate(n);
arr.sort((a: number, b: number): number => a - b);
console.log(checksum(arr));
