function sortTest(): void {
    let nums: number[] = [3, 1, 4, 1, 5, 9, 2, 6];
    nums.sort();
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
function run(): void {
    sortTest();
}
run();
