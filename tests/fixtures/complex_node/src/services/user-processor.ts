import { HttpClient } from '../utils/http-client';

export interface User { name: string; }

export class UserProcessor {
    private client: HttpClient<User>;

    constructor() {
        this.client = new HttpClient<User>("https://api.users.com");
    }

    async process(id: string): Promise<string> {
        const user = await this.client.get("/" + id);
        return user.name.trim().toUpperCase();
    }
}
