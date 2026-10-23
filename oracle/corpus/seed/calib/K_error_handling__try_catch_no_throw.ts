function safe(): string {
    try {
        const x: number = 42;
        return x.toString();
    } catch (error) {
        return "error";
    }
}

function run(): void {
    console.log(safe());
}
run();
