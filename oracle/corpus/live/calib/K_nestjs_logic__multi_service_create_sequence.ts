class UsersService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    create(name: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "User " + name + " created with id " + id.toString();
    }

    findAll(): string {
        return "[]";
    }
}

class AppService {
    getHealth(): string {
        return "ok";
    }
}

function run(): void {
    let usersSvc = new UsersService();
    let appSvc = new AppService();
    console.log(appSvc.getHealth());
    console.log(usersSvc.create("Alice"));
    console.log(usersSvc.create("Bob"));
    console.log(usersSvc.findAll());
}
run();
