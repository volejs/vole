const pattern = new URLPattern("/users/:id");
console.log("test /users/42:", pattern.test("https://example.com/users/42"));
console.log("test /posts/1:", pattern.test("https://example.com/posts/1"));
const match = pattern.exec("https://example.com/users/42");
console.log("match id:", match?.pathname?.groups?.id);
