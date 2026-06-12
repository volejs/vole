import { test } from "std:test";

test("RSASSA-PKCS1-v1.5 generateKey, sign and verify", async () => {
  const algorithm = {
    name: "RSASSA-PKCS1-v1.5",
    modulusLength: 2048,
    publicExponent: new Uint8Array([1, 0, 1]),
    hash: { name: "SHA-256" },
  };
  const keyPair = await crypto.subtle.generateKey(algorithm, true, [
    "sign",
    "verify",
  ]);
  if (!keyPair.privateKey || !keyPair.publicKey) {
    throw new Error("generateKey did not return a key pair");
  }
  if (keyPair.privateKey.type !== "private") {
    throw new Error("privateKey type mismatch: " + keyPair.privateKey.type);
  }
  if (keyPair.publicKey.type !== "public") {
    throw new Error("publicKey type mismatch: " + keyPair.publicKey.type);
  }

  const data = new TextEncoder().encode("hello rsa");
  const signature = await crypto.subtle.sign(
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    keyPair.privateKey,
    data
  );
  if (!(signature instanceof ArrayBuffer) || signature.byteLength === 0) {
    throw new Error("sign did not return a signature");
  }

  const valid = await crypto.subtle.verify(
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    keyPair.publicKey,
    signature,
    data
  );
  if (valid !== true) {
    throw new Error("verify returned false for valid signature");
  }

  const tampered = new TextEncoder().encode("hello rsa!");
  const invalid = await crypto.subtle.verify(
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    keyPair.publicKey,
    signature,
    tampered
  );
  if (invalid !== false) {
    throw new Error("verify returned true for tampered data");
  }
});

test("RSA-PSS generateKey, sign and verify", async () => {
  const algorithm = {
    name: "RSA-PSS",
    modulusLength: 2048,
    publicExponent: new Uint8Array([1, 0, 1]),
    hash: { name: "SHA-256" },
  };
  const keyPair = await crypto.subtle.generateKey(algorithm, true, [
    "sign",
    "verify",
  ]);
  if (!keyPair.privateKey || !keyPair.publicKey) {
    throw new Error("generateKey did not return a key pair");
  }

  const data = new TextEncoder().encode("hello pss");
  const signature = await crypto.subtle.sign(
    { name: "RSA-PSS", hash: { name: "SHA-256" } },
    keyPair.privateKey,
    data
  );

  const valid = await crypto.subtle.verify(
    { name: "RSA-PSS", hash: { name: "SHA-256" } },
    keyPair.publicKey,
    signature,
    data
  );
  if (valid !== true) {
    throw new Error("verify returned false for valid PSS signature");
  }
});

test("RSASSA-PKCS1-v1.5 import/export pkcs8 and spki", async () => {
  const algorithm = {
    name: "RSASSA-PKCS1-v1.5",
    modulusLength: 2048,
    publicExponent: new Uint8Array([1, 0, 1]),
    hash: { name: "SHA-256" },
  };
  const keyPair = await crypto.subtle.generateKey(algorithm, true, [
    "sign",
    "verify",
  ]);

  const privateDer = await crypto.subtle.exportKey("pkcs8", keyPair.privateKey);
  const publicDer = await crypto.subtle.exportKey("spki", keyPair.publicKey);

  if (!(privateDer instanceof ArrayBuffer)) {
    throw new Error("pkcs8 export did not return ArrayBuffer");
  }
  if (!(publicDer instanceof ArrayBuffer)) {
    throw new Error("spki export did not return ArrayBuffer");
  }

  const importedPrivate = await crypto.subtle.importKey(
    "pkcs8",
    privateDer,
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    true,
    ["sign"]
  );
  const importedPublic = await crypto.subtle.importKey(
    "spki",
    publicDer,
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    true,
    ["verify"]
  );

  if (importedPrivate.type !== "private") {
    throw new Error("imported private key type mismatch");
  }
  if (importedPublic.type !== "public") {
    throw new Error("imported public key type mismatch");
  }

  const data = new TextEncoder().encode("roundtrip");
  const signature = await crypto.subtle.sign(
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    importedPrivate,
    data
  );
  const valid = await crypto.subtle.verify(
    { name: "RSASSA-PKCS1-v1.5", hash: { name: "SHA-256" } },
    importedPublic,
    signature,
    data
  );
  if (valid !== true) {
    throw new Error("verify failed after import/export roundtrip");
  }
});
