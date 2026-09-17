let ctx = null;
let muted = false;
let master = null;

function ac() {
  if (typeof AudioContext === 'undefined' && typeof webkitAudioContext === 'undefined') return null;
  if (!ctx) {
    const AC = AudioContext || webkitAudioContext;
    ctx = new AC();
    master = ctx.createGain();
    master.gain.value = 0.28;
    master.connect(ctx.destination);
  }
  if (ctx.state === 'suspended') ctx.resume();
  return ctx;
}

export function setMuted(value) {
  muted = Boolean(value);
  if (master) master.gain.value = muted ? 0 : 0.28;
}

export function unlockAudio() {
  ac();
}

function envGain(c, start, peak, attack, decay) {
  const g = c.createGain();
  g.gain.setValueAtTime(0.0001, start);
  g.gain.exponentialRampToValueAtTime(peak, start + attack);
  g.gain.exponentialRampToValueAtTime(0.0001, start + attack + decay);
  return g;
}

function tone(freq, dur, type = 'square', peak = 0.2, slide = 0) {
  if (muted) return;
  const c = ac();
  if (!c) return;
  const t = c.currentTime;
  const osc = c.createOscillator();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t);
  if (slide) osc.frequency.exponentialRampToValueAtTime(Math.max(40, freq + slide), t + dur);
  const g = envGain(c, t, peak, 0.01, dur);
  osc.connect(g);
  g.connect(master);
  osc.start(t);
  osc.stop(t + dur + 0.02);
}

function noise(dur, peak = 0.12, hp = 800) {
  if (muted) return;
  const c = ac();
  if (!c) return;
  const t = c.currentTime;
  const n = Math.floor(c.sampleRate * dur);
  const buf = c.createBuffer(1, n, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < n; i += 1) data[i] = Math.random() * 2 - 1;
  const src = c.createBufferSource();
  src.buffer = buf;
  const filter = c.createBiquadFilter();
  filter.type = 'highpass';
  filter.frequency.value = hp;
  const g = envGain(c, t, peak, 0.005, dur);
  src.connect(filter);
  filter.connect(g);
  g.connect(master);
  src.start(t);
  src.stop(t + dur + 0.02);
}

export function scoopSound(crit = false, rarity = 'common') {
  noise(0.06, crit ? 0.16 : 0.08, 600);
  tone(crit ? 420 : 240, 0.08, 'square', 0.14, 180);
  if (rarity === 'rare' || rarity === 'epic') tone(520, 0.12, 'triangle', 0.12, 200);
  if (rarity === 'legendary' || rarity === 'mythic') fanfare();
}

export function coinSound() {
  tone(880, 0.08, 'square', 0.12, 200);
  setTimeout(() => tone(1240, 0.1, 'square', 0.1, 80), 50);
}

export function buySound() {
  tone(196, 0.07, 'square', 0.16);
  setTimeout(() => tone(247, 0.07, 'square', 0.14), 60);
  setTimeout(() => tone(330, 0.12, 'square', 0.12), 120);
}

export function fanfare() {
  tone(392, 0.1, 'square', 0.16);
  setTimeout(() => tone(523, 0.1, 'square', 0.14), 80);
  setTimeout(() => tone(659, 0.18, 'square', 0.14), 160);
}

export function denySound() {
  tone(120, 0.12, 'sawtooth', 0.1, -40);
}

export function petSound() {
  tone(660, 0.08, 'sine', 0.12, 120);
  setTimeout(() => tone(880, 0.1, 'sine', 0.1), 70);
}

export function kickSound() {
  noise(0.1, 0.2, 200);
  tone(90, 0.14, 'sine', 0.2, -30);
}

export function travelSound() {
  tone(220, 0.12, 'triangle', 0.14, 80);
  setTimeout(() => tone(330, 0.12, 'triangle', 0.12, 80), 100);
  setTimeout(() => tone(440, 0.18, 'triangle', 0.12, 120), 200);
}
