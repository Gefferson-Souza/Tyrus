function run(): void {
    const a: number[] = [1, 2];
    const b: number[] = [3, 4];
    const c: number[] = a.concat(b);
    c.forEach((n: number): void => {
        console.log(n);
    });
}
run();
