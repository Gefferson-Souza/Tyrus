function run(): void {
    let nums: number[] = [1, 2, 3, 4, 5];
    nums.reverse();
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
run();
