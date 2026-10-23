function shiftTest(): void {
    let items: number[] = [10, 20, 30];
    const first: number = items.shift();
    console.log(first);
    items.forEach((n: number): void => {
        console.log(n);
    });
}
function run(): void {
    shiftTest();
}
run();
