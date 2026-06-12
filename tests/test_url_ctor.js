try {
    const u = new URL("https://example.com/path");
    console.log("URL href:", u.href);
} catch (e) {
    console.log("URL error:", e.message);
}
