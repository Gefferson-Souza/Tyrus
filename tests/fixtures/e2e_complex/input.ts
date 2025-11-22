// Complex E2E scenario combining all Milestone features
// Milestone 2: Interfaces (DTOs)
interface User {
    id: number;
    name: string;
    email: string;
    isActive: boolean;
}

interface ApiResponse {
    success: boolean;
    data: string;
    timestamp: number;
}

// Milestone 3: Functions with Math
function calculateAge(birthYear: number, currentYear: number): number {
    return currentYear - birthYear;
}

function sum(a: number, b: number, c: number): number {
    return a + b + c;
}

// Helper functions for database operations
async function getFromDatabase(id: number): Promise<User> {
    // Simulated database fetch
    return {
        id: id,
        name: "Test User",
        email: "test@example.com",
        isActive: true
    };
}

async function postToDatabase(user: User): Promise<ApiResponse> {
    // Simulated database save
    return {
        success: true,
        data: `User ${user.name} saved`,
        timestamp: Date.now()
    };
}

// Milestone 4: Async/Await
async function fetchUser(id: number): Promise<User> {
    return await getFromDatabase(id);
}

async function saveUser(user: User): Promise<ApiResponse> {
    return await postToDatabase(user);
}

// Mixed sync/async
function processUser(user: User): number {
    return calculateAge(user.id, 2024);
}
