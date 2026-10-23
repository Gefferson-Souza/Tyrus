class UserStore {
    findAll(): string {
        return "[]";
    }
}

class UserHandler {
    private store: UserStore;

    constructor(store: UserStore) {
        this.store = store;
    }

    getAll(): string {
        return this.store.findAll();
    }
}

function run(): void {
    let store = new UserStore();
    let handler = new UserHandler(store);
    console.log(handler.getAll());
}
run();
