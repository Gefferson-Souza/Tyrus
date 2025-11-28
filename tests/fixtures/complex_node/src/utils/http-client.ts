import axios from 'axios';

export class HttpClient<T> {
    constructor(private baseUrl: string) { }

    async get(path: string): Promise<T> {
        // Test Generic Return + Axios
        return await axios.get<T>(this.baseUrl + path);
    }
}
