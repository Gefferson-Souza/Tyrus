function trig(n: number): number {
    let acc: number = 0;
    for (let i: number = 1; i <= n; i++) {
        acc = acc + Math.sin(i * 0.001) * Math.cos(i * 0.002) + Math.sqrt(i) + Math.log(i);
    }
    return Math.floor(acc);
}
console.log(trig(20000000));
