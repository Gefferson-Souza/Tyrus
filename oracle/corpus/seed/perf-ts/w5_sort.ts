function generate(n: number): number[] {
    const out: number[] = [];
    let seed: number = 12345;
    for (let i: number = 0; i < n; i++) {
        seed = (seed * 1664525 + 1013904223) % 4294967296;
        out.push(seed);
    }
    return out;
}
const n: number = 1000000;
const arr: number[] = generate(n);
arr.sort((a: number, b: number): number => a - b);
let check: number = 0;
for (let i: number = 0; i < n; i += 1000) {
    check = (check + arr[i]) % 1000000007;
}
console.log(arr[0]);
console.log(arr[500000]);
console.log(arr[n - 1]);
console.log(check);
