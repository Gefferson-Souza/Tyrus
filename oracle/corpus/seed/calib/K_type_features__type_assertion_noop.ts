function getUser(): string {
    const data: string = "Alice";
    return data;
}
function run(): void {
    const user: string = getUser();
    console.log(user);
}
run();
