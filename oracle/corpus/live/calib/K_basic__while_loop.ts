function sum(n: number): number {
    let total: number = 0;
    let i: number = 1;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
function run(): void {
    console.log(sum(5));
    console.log(sum(10));
}
run();
