class UsersService {
    findAll(): string {
        return "[]";
    }
}

function run(): void {
    let svc = new UsersService();
    console.log(svc.findAll());
}
run();
