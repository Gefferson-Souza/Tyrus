class Counter {
    private count: number = 0;
    increment(): void {
        this.count = this.count + 1;
    }
    getCount(): number {
        return this.count;
    }
}
let c: Counter = new Counter();
const n: number = 200000000;
for (let i: number = 0; i < n; i++) {
    c.increment();
}
console.log(c.getCount());
