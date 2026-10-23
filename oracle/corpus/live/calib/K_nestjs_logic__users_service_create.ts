class UsersService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    create(name: string, email: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "{\"id\":" + id.toString() + ",\"name\":\"" + name + "\",\"email\":\"" + email + "\"}";
    }
}

function run(): void {
    let svc = new UsersService();
    console.log(svc.create("Alice", "alice@test.com"));
    console.log(svc.create("Bob", "bob@test.com"));
}
run();
