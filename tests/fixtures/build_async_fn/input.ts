async function fetchData(url: string): Promise<string> {
    return await request(url);
}

function simpleCall(a: number): number {
    return process(a);
}
