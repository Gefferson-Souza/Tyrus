function safeDivide(a: number, b: number): string {
    try {
        if (b === 0) {
            throw new Error("Division by zero");
        }
        const result: number = a / b;
        return result.toString();
    } catch (error) {
        return "Error caught";
    }
}

function run(): void {
    console.log(safeDivide(10, 2));
    console.log(safeDivide(10, 0));
}
run();
