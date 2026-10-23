class CounterService {
    private count: number = 0;
    increment(): void {
        this.count = this.count + 1;
    }
    getCount(): number {
        return this.count;
    }
}
const svc: CounterService = new CounterService();
const n: number = 10000000;
for (let i: number = 0; i < n; i++) {
    svc.increment();
}
console.log(svc.getCount());
