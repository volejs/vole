async function main() {
    for (let i = 1; i <= 5; i++) {
        await new Promise(r => setTimeout(r, 5));
        console.log("step", i);
    }
}
main();
