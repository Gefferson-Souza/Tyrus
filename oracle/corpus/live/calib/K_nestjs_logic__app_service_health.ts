class AppService {
    getHealth(): string {
        return "{\"status\":\"ok\"}";
    }
}

function run(): void {
    let svc = new AppService();
    console.log(svc.getHealth());
}
run();
