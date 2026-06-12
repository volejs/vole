console.log("canParse valid:", URL.canParse("https://example.com/path"));
console.log("canParse invalid:", URL.canParse("not a url"));
console.log("parse valid:", URL.parse("https://example.com/path")?.href);
console.log("parse invalid:", URL.parse("not a url"));
