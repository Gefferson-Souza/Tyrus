function outer(): string {
    try {
        try {
            throw new Error("inner");
        } catch (e) {
            return "caught inner";
        }
    } catch (e) {
        return "caught outer";
    }
    return "no error";
}

function run(): void {
    console.log(outer());
}
run();
