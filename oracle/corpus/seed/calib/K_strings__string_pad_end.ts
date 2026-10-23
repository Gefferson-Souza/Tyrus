function padEndTest(): string {
    const s: string = "hello";
    return s.padEnd(10, "-");
}
function run(): void {
    console.log(padEndTest());
}
run();
