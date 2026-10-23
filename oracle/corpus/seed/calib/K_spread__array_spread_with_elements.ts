function build(): string {
    const middle: string[] = ["b", "c"];
    const all: string[] = ["a", ...middle, "d"];
    return all.join(", ");
}
console.log(build());
