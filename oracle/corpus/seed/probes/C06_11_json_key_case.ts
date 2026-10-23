interface User { user_name: string; firstName: string; }
const u: User = { user_name: "a", firstName: "b" };
console.log(JSON.stringify(u));
