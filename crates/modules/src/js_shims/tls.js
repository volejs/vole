const API = {
  enabled: false,
};

export function isSupported() {
  return API.enabled;
}

export function isAvailable() {
  return API.enabled;
}

export function getInfo() {
  return {
    enabled: API.enabled,
  };
}

export const tls = {
  isSupported,
  isAvailable,
  getInfo,
};

export default {
  isSupported,
  isAvailable,
  getInfo,
};
