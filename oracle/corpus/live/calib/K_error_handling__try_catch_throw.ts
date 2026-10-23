function processAge(age: number): string {
    try {
        if (age < 0) {
            throw new Error("Age cannot be negative");
        }
        if (age > 150) {
            throw new Error("Age too high");
        }
        return "Valid age: " + age.toString();
    } catch (error) {
        return "Invalid: caught error";
    }
}

function run(): void {
    console.log(processAge(25));
    console.log(processAge(-5));
    console.log(processAge(200));
}
run();
