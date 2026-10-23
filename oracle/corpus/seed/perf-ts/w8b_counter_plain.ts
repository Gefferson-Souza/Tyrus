class Counter {
    private count: number = 0;
    increment(): void {
        this.count = this.count + 1;
    }
    getCount(): number {
        return this.count;
    }
}
const c: Counter = new Counter();
const n: number = 10000000;
for (let i: number = 0; i < n; i++) {
    c.increment();
}
console.log(c.getCount());
