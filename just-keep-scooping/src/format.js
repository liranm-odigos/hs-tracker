const SUFFIXES = ['', 'K', 'M', 'B', 'T', 'Qa', 'Qi', 'Sx', 'Sp', 'Oc', 'No', 'Dc'];

export function formatNumber(n) {
  if (!Number.isFinite(n)) return '0';
  const sign = n < 0 ? '-' : '';
  let v = Math.abs(n);
  if (v < 1000) return sign + Math.floor(v).toString();
  let e = 0;
  while (v >= 1000 && e < SUFFIXES.length - 1) {
    v /= 1000;
    e += 1;
  }
  const digits = v >= 100 ? 0 : v >= 10 ? 1 : 2;
  const text = v.toFixed(digits).replace(/\.0+$/, '').replace(/(\.\d*[1-9])0+$/, '$1');
  return sign + text + SUFFIXES[e];
}

export function formatCoins(n) {
  return formatNumber(n) + '¢';
}

export function formatPct(n) {
  return Math.round(n * 100) + '%';
}

export function formatMs(ms) {
  if (ms < 1000) return Math.round(ms) + 'ms';
  return (ms / 1000).toFixed(2) + 's';
}
