try {
    var u = new URL("https://example.com/path");
    console.log("ctor ok:", u.href);
} catch (e) {
    console.log("ctor error:", e.message);
}

try {
    var v = new URL("https://example.com/path", undefined);
    console.log("ctor with undefined ok:", v.href);
} catch (e) {
    console.log("ctor with undefined error:", e.message);
}

console.log("canParse result:", URL.canParse("https://example.com/path"));
