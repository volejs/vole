const fd = new FormData();
fd.append("foo", "bar");
fd.append("foo", "baz");
fd.set("qux", "quux");
console.log("FormData get:", fd.get("foo"));
console.log("FormData getAll:", fd.getAll("foo").join(","));
console.log("FormData has:", fd.has("qux"));
fd.delete("foo");
console.log("FormData after delete:", fd.has("foo"));
for (const [k, v] of fd) {
    console.log("FormData entry:", k, v);
}
