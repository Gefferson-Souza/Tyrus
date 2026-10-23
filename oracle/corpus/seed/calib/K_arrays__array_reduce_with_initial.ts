function run(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    const sum: number = nums.reduce((acc: number, n: number): number => acc + n, 0);
    console.log(sum);
}
run();
