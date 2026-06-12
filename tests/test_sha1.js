async function main() {
    const data = new TextEncoder().encode("hello");
    const hash = await crypto.subtle.digest("SHA-1", data);
    const hex = Array.from(new Uint8Array(hash))
        .map(b => b.toString(16).padStart(2, "0"))
        .join("");
    console.log("SHA-1:", hex);
    // expected: aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
}
main();
