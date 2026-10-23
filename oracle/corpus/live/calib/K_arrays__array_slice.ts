function run(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    const sliced: number[] = nums.slice(1, 4);
    sliced.forEach((n: number): void => {
        console.log(n);
    });
}
run();
