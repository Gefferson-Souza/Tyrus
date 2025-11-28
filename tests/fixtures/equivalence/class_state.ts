// Tests Class internal state mutation (this.x = ...)
class Counter {
    count: number;
    constructor(start: number) { this.count = start; }

    increment(amount: number) {
        this.count = this.count + amount;
    }

    report(): string {
        return "Final: " + this.count;
    }
}

const c = new Counter(10);
c.increment(5);
c.increment(20);
console.log(c.report());
