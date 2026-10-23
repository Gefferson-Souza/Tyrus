function fill(n: number): Map<string, number> {
    const m: Map<string, number> = new Map<string, number>();
    for (let i: number = 0; i < n; i++) {
        m.set("key" + i, i * 2);
    }
    return m;
}
function lookupAll(m: Map<string, number>, n: number): number {
    let sum: number = 0;
    for (let i: number = 0; i < n; i++) {
        const v: number | undefined = m.get("key" + i);
        if (v !== undefined) {
            sum += v;
        }
    }
    return sum;
}
const n: number = 500000;
const m: Map<string, number> = fill(n);
console.log(m.size);
console.log(lookupAll(m, n));
