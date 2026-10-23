function dirName(d: number): string {
    if (d === 0) { return "Up"; }
    if (d === 1) { return "Down"; }
    if (d === 2) { return "Left"; }
    return "Right";
}
function run(): void {
    console.log(dirName(0));
    console.log(dirName(1));
    console.log(dirName(3));
}
run();
