const KEY = 'just-keep-scooping-v1';

export function loadSave() {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function writeSave(state) {
  try {
    const copy = { ...state, derived: null };
    localStorage.setItem(KEY, JSON.stringify(copy));
  } catch {
    // quota / private mode
  }
}

export function clearSave() {
  try {
    localStorage.removeItem(KEY);
  } catch {
    // ignore
  }
}

export function exportSave(state) {
  return btoa(unescape(encodeURIComponent(JSON.stringify({ ...state, derived: null }))));
}

export function importSave(text) {
  const json = decodeURIComponent(escape(atob(text.trim())));
  return JSON.parse(json);
}
