function padStartTest(): string {
    const s: string = "hello";
    return s.padStart(10, "*");
}
function run(): void {
    console.log(padStartTest());
}
run();
