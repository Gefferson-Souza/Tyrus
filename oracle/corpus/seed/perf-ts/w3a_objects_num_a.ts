interface Vec2 {
    x: number;
    y: number;
    weight: number;
}
function makeAll(n: number): Vec2[] {
    let out: Vec2[] = [];
    for (let i: number = 0; i < n; i++) {
        const p: Vec2 = { x: i, y: i * 2, weight: i % 10 };
        out.push(p);
    }
    return out;
}
function sumFields(items: Vec2[]): number {
    let sum: number = 0;
    for (const it of items) {
        sum += it.x + it.weight;
    }
    return sum;
}
const n: number = 1000000;
const rounds: number = 5;
let total: number = 0;
for (let r: number = 0; r < rounds; r++) {
    total += sumFields(makeAll(n));
}
console.log(total);
