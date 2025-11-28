// Tests map, filter, string manipulation, and object structure
const rawData = ["User:Alice:Active", "User:Bob:Inactive", "Admin:Charlie:Active"];

const processed = rawData
    .map((line) => {
        const parts = line.split(":");
        return { role: parts[0], name: parts[1], status: parts[2] };
    })
    .filter((u) => u["status"] === "Active")
    .map((u) => {
        return {
            role: u["role"],
            name: u["name"],
            status: u["status"],
            id: "ID_123"
        };
    }); // Object spread simulation if supported, else manual

console.log(JSON.stringify(processed));
