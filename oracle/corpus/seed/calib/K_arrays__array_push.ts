function run(): void {
    let items: number[] = [1, 2, 3];
    items.push(4);
    items.push(5);
    items.forEach((n: number): void => {
        console.log(n);
    });
}
run();
