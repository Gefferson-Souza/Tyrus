export type ID = string;
export type NumberList = number[];

export interface Container {
    id: ID;
    values: number[];
    optional?: string;
    maybe: number | undefined;
}

export function processList(list: number[]): number {
    // Simple sum for now, assuming we can handle basic iteration or just return length for testing
    // Since we don't have full array method support yet (reduce), let's just return the length cast to number
    // or use a loop if we implemented for-of (we haven't explicitly yet, but we have basic for loops?)
    // Let's keep it simple: return 42.0 for now to test signature, or implement a basic loop if supported.
    // Actually, let's just return list.length if we map it to Vec.
    // But `list.length` access needs MemberExpr support which we have.
    // Vec::len() returns usize, we need f64.
    // Let's just return a constant for the first pass to verify type compilation.
    return 42;
}
