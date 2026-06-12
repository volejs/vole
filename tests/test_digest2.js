async function main() {
    const data = new TextEncoder().encode("hello");
    for (const algo of ["SHA-1", "SHA-256", "SHA-384", "SHA-512"]) {
        try {
            const hash = await crypto.subtle.digest(algo, data);
            const hex = Array.from(new Uint8Array(hash))
                .map(b => b.toString(16).padStart(2, "0"))
                .join("");
            console.log(algo + ":", hex.length, "chars");
        } catch (e) {
            console.log("ERROR", algo, e.message || e);
        }
    }
}
main();
