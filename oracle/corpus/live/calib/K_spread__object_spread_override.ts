interface Config {
    host: string;
    port: number;
    debug: boolean;
}
function createProd(base: Config): Config {
    const prod: Config = { ...base, debug: false };
    return prod;
}
function run(): void {
    const dev: Config = { host: "localhost", port: 3100, debug: true };
    const prod: Config = createProd(dev);
    console.log(prod.host);
    console.log(prod.port);
    console.log(prod.debug);
}
run();
