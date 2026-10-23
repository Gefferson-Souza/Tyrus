class MathUtils {
    static add(a: number, b: number): number {
        return a + b;
    }
    static multiply(a: number, b: number): number {
        return a * b;
    }
}

function run(): void {
    console.log(MathUtils.add(3, 4));
    console.log(MathUtils.multiply(5, 6));
}
run();
