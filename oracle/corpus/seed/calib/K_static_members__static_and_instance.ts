class Counter {
    count: number;
    constructor() {
        this.count = 0;
    }
    static create(): Counter {
        return new Counter();
    }
    increment(): number {
        this.count = this.count + 1;
        return this.count;
    }
}

function run(): void {
    let c = Counter.create();
    console.log(c.increment());
    console.log(c.increment());
}
run();
