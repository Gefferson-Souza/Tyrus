function run(n: number): number {
    let acc: number = 0;
    for (let i: number = 1; i <= n; i++) {
        for (let j: number = 1; j <= n; j++) {
            if ((i * j) % 7 === 3 || (i + j) % 11 === 0) {
                acc += (i * j) % 13;
            }
        }
    }
    return acc;
}
console.log(run(12000));
