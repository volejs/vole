async function main() {
  try {
    // AES-CBC JWK round-trip
    const aesKey = await crypto.subtle.generateKey(
      { name: "AES-CBC", length: 256 },
      true,
      ["encrypt", "decrypt"]
    );
    const aesJwk = await crypto.subtle.exportKey("jwk", aesKey);
    console.log("AES-CBC JWK:", aesJwk.kty, aesJwk.alg, aesJwk.k.length);
    if (aesJwk.kty !== "oct" || aesJwk.alg !== "A256CBC") {
      throw new Error("AES-CBC JWK export failed");
    }
    const aesImported = await crypto.subtle.importKey(
      "jwk",
      aesJwk,
      { name: "AES-CBC" },
      true,
      ["encrypt", "decrypt"]
    );
    console.log("AES-CBC import ok, id:", aesImported._id);

    // HMAC JWK round-trip
    const hmacKey = await crypto.subtle.generateKey(
      { name: "HMAC", hash: "SHA-256" },
      true,
      ["sign", "verify"]
    );
    const hmacJwk = await crypto.subtle.exportKey("jwk", hmacKey);
    console.log("HMAC JWK:", hmacJwk.kty, hmacJwk.alg, hmacJwk.k.length);
    if (hmacJwk.kty !== "oct" || hmacJwk.alg !== "HS256") {
      throw new Error("HMAC JWK export failed");
    }
    const hmacImported = await crypto.subtle.importKey(
      "jwk",
      hmacJwk,
      { name: "HMAC", hash: "SHA-256" },
      true,
      ["sign", "verify"]
    );
    console.log("HMAC import ok, id:", hmacImported._id);

    // Verify imported HMAC key still works
    const data = new TextEncoder().encode("hello jwk");
    console.log("signing...");
    const sig = await crypto.subtle.sign("HMAC", hmacImported, data);
    console.log("signature length:", sig.byteLength);
    console.log("verifying...");
    const ok = await crypto.subtle.verify("HMAC", hmacImported, sig, data);
    console.log("HMAC verify:", ok);
    if (!ok) {
      throw new Error("Imported HMAC key verify failed");
    }

    // Verify imported AES key still works
    const iv = crypto.getRandomValues(new Uint8Array(16));
    console.log("encrypting...");
    const encrypted = await crypto.subtle.encrypt({ name: "AES-CBC", iv }, aesImported, data);
    console.log("encrypted length:", encrypted.byteLength);
    console.log("decrypting...");
    const decrypted = await crypto.subtle.decrypt({ name: "AES-CBC", iv }, aesImported, encrypted);
    const decryptedText = new TextDecoder().decode(decrypted);
    console.log("AES decrypt:", decryptedText);
    if (decryptedText !== "hello jwk") {
      throw new Error("Imported AES key decrypt failed");
    }

    console.log("=== crypto JWK test passed ===");
  } catch (e) {
    console.log("ERROR:", e && e.message ? e.message : e);
  }
}

main();
