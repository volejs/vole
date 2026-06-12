async function main() {
    const data = new TextEncoder().encode("hello");
    try {
        const hash = await crypto.subtle.digest("SHA-512", data);
        const hex = Array.from(new Uint8Array(hash))
            .map(b => b.toString(16).padStart(2, "0"))
            .join("");
        console.log("SHA-512:", hex.length, "chars");
    } catch (e) {
        console.log("error:", e.message || e);
    }
}
main();
