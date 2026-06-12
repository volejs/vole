import {
  __crypto_generate_key,
  __crypto_encrypt,
  __crypto_decrypt,
  __crypto_sign,
  __crypto_verify,
  __crypto_import_key,
  __crypto_export_key,
} from "std:raw:crypto";

const originalDigest = crypto.subtle.digest.bind(crypto.subtle);
crypto.subtle.digest = function (algorithm, data) {
  if (data instanceof DataView) {
    data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  return originalDigest(algorithm, data);
};

class CryptoKey {
  constructor(id, type, extractable, algorithm, usages) {
    this._id = id;
    this.type = type;
    this.extractable = extractable;
    this.algorithm = algorithm;
    this.usages = usages;
  }
}

globalThis.CryptoKey = CryptoKey;

function makeCryptoKey(key, algorithm) {
  try {
    if (key && typeof key === "object") {
      key.algorithm = algorithm;
    }
  } catch (e) {
    console.error("makeCryptoKey error:", e && (e.stack || e.message || e));
  }
  return key;
}

function serializeAlgorithm(alg) {
  const copy = {};
  for (const key in alg) {
    if (alg[key] instanceof Uint8Array || alg[key] instanceof Int8Array) {
      copy[key] = Array.from(alg[key]);
    } else {
      copy[key] = alg[key];
    }
  }
  return JSON.stringify(copy);
}

crypto.subtle.generateKey = function (algorithm, extractable, keyUsages) {
  return __crypto_generate_key(JSON.stringify(algorithm), extractable, keyUsages).then(function (result) {
    if (result && result.privateKey && result.publicKey) {
      makeCryptoKey(result.privateKey, algorithm);
      makeCryptoKey(result.publicKey, algorithm);
    } else {
      makeCryptoKey(result, algorithm);
    }
    return result;
  });
};

crypto.subtle.encrypt = function (algorithm, key, data) {
  if (data instanceof DataView) {
    data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  return __crypto_encrypt(serializeAlgorithm(algorithm), key._id, data);
};

crypto.subtle.decrypt = function (algorithm, key, data) {
  if (data instanceof DataView) {
    data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  return __crypto_decrypt(serializeAlgorithm(algorithm), key._id, data);
};

crypto.subtle.sign = function (algorithm, key, data) {
  if (data instanceof DataView) {
    data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  return __crypto_sign(JSON.stringify(algorithm), key._id, data);
};

crypto.subtle.verify = function (algorithm, key, signature, data) {
  if (data instanceof DataView) {
    data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  }
  if (signature instanceof ArrayBuffer) {
    signature = new Uint8Array(signature);
  }
  return __crypto_verify(JSON.stringify(algorithm), key._id, signature, data);
};

crypto.subtle.importKey = function (format, keyData, algorithm, extractable, keyUsages) {
  let data = keyData;
  if (format === "jwk") {
    data = JSON.stringify(keyData);
  }
  const key = __crypto_import_key(format, data, JSON.stringify(algorithm), extractable, keyUsages);
  makeCryptoKey(key, algorithm);
  return Promise.resolve(key);
};

crypto.subtle.exportKey = function (format, key) {
  return Promise.resolve(__crypto_export_key(format, key._id));
};

globalThis.Crypto = function () { };
globalThis.SubtleCrypto = function SubtleCrypto() { };
crypto.subtle.constructor = SubtleCrypto;
Object.setPrototypeOf(crypto.subtle, SubtleCrypto.prototype);

export default crypto;
