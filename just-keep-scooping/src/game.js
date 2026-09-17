import {
  ACHIEVEMENTS,
  BUFFS,
  CREW,
  LOCATIONS,
  RARITY_RANK,
  TRASH,
  UPGRADES,
  poolFor,
} from './data.js';

const COMBO_WINDOW = 1600;
const BUFF_IDS = Object.keys(BUFFS);

export function createState() {
  return {
    coins: 0,
    lifetimeCoins: 0,
    bag: [],
    bottlecaps: 0,
    locationIndex: 0,
    upgrades: {},
    crew: {},
    discovered: [],
    achievements: [],
    stats: {
      scoops: 0,
      items: 0,
      sold: 0,
      crits: 0,
      maxCombo: 0,
      kicks: 0,
      pets: 0,
      travels: 0,
      stuffed: 0,
      rares: 0,
      epics: 0,
      legendaries: 0,
      mythics: 0,
      playMs: 0,
    },
    infamy: 1,
    prestigeCount: 0,
    combo: 0,
    comboAt: 0,
    cooldown: 0,
    buffs: [],
    catUntil: 0,
    catSpawnAt: 8000,
    possumUntil: 0,
    possumBlocking: false,
    internAcc: 0,
    muted: false,
    started: false,
    log: [],
    derived: null,
  };
}

export function hydrate(raw) {
  const s = createState();
  if (!raw || typeof raw !== 'object') return recompute(s);
  Object.assign(s, raw);
  s.upgrades = { ...(raw.upgrades || {}) };
  s.crew = { ...(raw.crew || {}) };
  s.stats = { ...createState().stats, ...(raw.stats || {}) };
  s.bag = Array.isArray(raw.bag) ? raw.bag.map((i) => ({ ...i })) : [];
  s.discovered = Array.isArray(raw.discovered) ? [...raw.discovered] : [];
  s.achievements = Array.isArray(raw.achievements) ? [...raw.achievements] : [];
  s.buffs = Array.isArray(raw.buffs) ? raw.buffs.map((b) => ({ ...b })) : [];
  s.log = Array.isArray(raw.log) ? [...raw.log] : [];
  return recompute(s);
}

export function getCost(def, level) {
  const rate = def.rate ?? 1.17;
  return Math.floor(def.baseCost * rate ** level * (1 + 0.035 * level));
}

export function isUnlocked(def, state) {
  return !def.unlock || def.unlock(state);
}

export function isMaxed(def, level) {
  return Boolean(def.max) && level >= def.max;
}

export function hasBuff(state, id) {
  return state.buffs.some((b) => b.id === id && b.until > 0);
}

export function nextLocation(state) {
  return LOCATIONS[state.locationIndex + 1] || null;
}

export function capsProgress(state) {
  const next = nextLocation(state);
  if (!next) return 1;
  if (next.capsNeeded <= 0) return 1;
  return Math.min(1, state.bottlecaps / next.capsNeeded);
}

export function canTravel(state) {
  recompute(state);
  const next = nextLocation(state);
  if (!next) return false;
  return state.bottlecaps >= next.capsNeeded && state.coins >= next.travelCost;
}

export function canPrestige(state) {
  return state.locationIndex >= 5 || state.lifetimeCoins >= 80000;
}

export function prestigeGain(state) {
  const loc = state.locationIndex;
  const money = Math.log10(Math.max(10, state.lifetimeCoins));
  return Number((0.4 + loc * 0.18 + money * 0.07).toFixed(2));
}

export function recompute(state) {
  const paw = state.upgrades.paw || 0;
  const bag = state.upgrades.bag || 0;
  const speed = state.upgrades.speed || 0;
  const value = state.upgrades.value || 0;
  const crit = state.upgrades.crit || 0;
  const luck = state.upgrades.luck || 0;
  const intern = state.crew.intern || 0;
  const accountant = state.crew.accountant || 0;
  const lookout = state.crew.lookout || 0;
  const union = state.crew.union || 0;
  const loc = LOCATIONS[state.locationIndex] || LOCATIONS[0];

  let scoopMs = 720 * 0.915 ** speed;
  if (hasBuff(state, 'frenzy')) scoopMs *= 0.42;
  scoopMs = Math.max(85, scoopMs);

  const unionMult = 1.12 ** union;
  let valueMult = (1 + value * 0.15) * unionMult * state.infamy * loc.mult;
  if (hasBuff(state, 'gold')) valueMult *= 3;

  const comboMult = 1 + Math.min(30, state.combo) * 0.03;
  const internMs =
    intern <= 0 ? Infinity : Math.max(280, 1900 / intern) * (scoopMs / 720);

  state.derived = {
    capacity: 8 + bag * 4,
    pawPower: 1 + paw + (hasBuff(state, 'magnet') ? 2 : 0),
    scoopMs,
    valueMult,
    comboMult,
    critChance: Math.min(0.45, 0.03 + crit * 0.025),
    critMult: 5,
    luck,
    lookout,
    intern,
    internMs,
    autoSell: accountant > 0,
    location: loc,
    bagFull: false,
  };
  state.derived.bagFull = state.bag.length >= state.derived.capacity;
  return state;
}

function pick(arr, rng) {
  if (!arr.length) return null;
  return arr[Math.floor(rng() * arr.length)];
}

export function rarityWeights(state) {
  recompute(state);
  const loc = state.locationIndex;
  const luck = (state.derived.luck || 0) + (state.derived.lookout || 0) * 0.7;
  const storm = hasBuff(state, 'luckstorm') ? 2.2 : 1;
  const bump = (1 + loc * 0.12 + luck * 0.16) * storm;

  let common = 72;
  let uncommon = 20;
  let rare = 6;
  let epic = 1.6;
  let legendary = 0.38;
  let mythic = loc >= 8 ? 0.12 : 0;

  const shift = Math.min(0.62, (bump - 1) * 0.18);
  const move = (from, to) => {
    const amt = from * shift;
    return [from - amt, to + amt];
  };
  [common, uncommon] = move(common, uncommon);
  [uncommon, rare] = move(uncommon, rare);
  [rare, epic] = move(rare, epic);
  [epic, legendary] = move(epic, legendary);
  if (loc >= 8) [legendary, mythic] = move(legendary, mythic);

  return { common, uncommon, rare, epic, legendary, mythic };
}

export function rollRarity(state, rng) {
  const w = rarityWeights(state);
  const total = Object.values(w).reduce((a, b) => a + b, 0);
  let r = rng() * total;
  for (const key of Object.keys(w)) {
    r -= w[key];
    if (r <= 0) return key;
  }
  return 'common';
}

export function rollItem(state, rng, bump = false) {
  recompute(state);
  let rarity = rollRarity(state, rng);
  if (bump) {
    const i = Math.min(RARITY_RANK[rarity] + 1, RARITY_RANK.mythic);
    rarity = Object.keys(RARITY_RANK).find((k) => RARITY_RANK[k] === i) || rarity;
    if (state.locationIndex < 8 && rarity === 'mythic') rarity = 'legendary';
  }
  const locId = state.derived.location.id;
  const { local, global } = poolFor(locId, rarity);
  let item = null;
  if (local.length && rng() < 0.72) item = pick(local, rng);
  if (!item) item = pick(global, rng) || pick(local, rng);
  if (!item) {
    const fallback = TRASH.filter((t) => t.rarity === 'common' && !t.loc);
    item = pick(fallback, rng) || TRASH[0];
  }
  return {
    id: item.id,
    name: item.name,
    rarity: item.rarity,
    value: item.value,
    icon: item.icon,
    quip: item.quip,
  };
}

export function itemValue(state, item) {
  recompute(state);
  return item.value * state.derived.valueMult * state.derived.comboMult;
}

function noteRarity(state, item) {
  if (item.rarity === 'rare') state.stats.rares += 1;
  if (item.rarity === 'epic') state.stats.epics += 1;
  if (item.rarity === 'legendary') state.stats.legendaries += 1;
  if (item.rarity === 'mythic') state.stats.mythics += 1;
}

function discover(state, item) {
  if (!state.discovered.includes(item.id)) state.discovered.push(item.id);
}

export function checkAchievements(state) {
  const unlocked = [];
  for (const a of ACHIEVEMENTS) {
    if (state.achievements.includes(a.id)) continue;
    if (a.test(state)) {
      state.achievements.push(a.id);
      unlocked.push(a);
    }
  }
  return unlocked;
}

function pushLog(state, text) {
  state.log.unshift(text);
  state.log = state.log.slice(0, 8);
}

function grantBuff(state, id, now) {
  const def = BUFFS[id];
  if (!def) return null;
  const existing = state.buffs.find((b) => b.id === id);
  const until = now + def.ms;
  if (existing) existing.until = until;
  else state.buffs.push({ id, until });
  return def;
}

function maybeWorld(state, rng, now) {
  if (!state.possumBlocking && now > state.possumUntil && state.stats.scoops > 12 && rng() < 0.04) {
    state.possumBlocking = true;
    state.possumUntil = now + 12000;
    pushLog(state, 'A possum sat on the lid. Workplace drama.');
  }
  if (now > state.catSpawnAt && rng() < 0.08) {
    state.catUntil = now + 7000;
    state.catSpawnAt = now + 18000 + rng() * 20000;
  }
}

export function scoop(state, rng = Math.random, now = Date.now()) {
  recompute(state);
  if (state.possumBlocking) return { ok: false, reason: 'possum' };
  if (state.cooldown > 0) return { ok: false, reason: 'cooldown' };

  const space = state.derived.capacity - state.bag.length;
  if (space <= 0) {
    state.stats.stuffed += 1;
    checkAchievements(state);
    return { ok: false, reason: 'full' };
  }

  state.cooldown = state.derived.scoopMs;
  if (now - state.comboAt <= COMBO_WINDOW) state.combo += 1;
  else state.combo = 1;
  state.comboAt = now;
  state.stats.maxCombo = Math.max(state.stats.maxCombo, state.combo);
  state.stats.scoops += 1;

  const crit = rng() < state.derived.critChance;
  if (crit) state.stats.crits += 1;

  let n = state.derived.pawPower;
  if (crit) n += 2;
  n = Math.max(1, Math.min(n, space));

  const items = [];
  for (let i = 0; i < n; i += 1) {
    const item = rollItem(state, rng, crit && i === 0);
    state.bag.push(item);
    items.push(item);
    discover(state, item);
    noteRarity(state, item);
  }
  state.stats.items += items.length;

  let caps = 0;
  if (nextLocation(state) && rng() < 0.26) {
    caps = 1 + (crit ? 1 : 0) + (hasBuff(state, 'magnet') ? 1 : 0);
    state.bottlecaps += caps;
  }

  let buff = null;
  if (state.stats.scoops > 8 && rng() < 0.038) {
    const id = BUFF_IDS[Math.floor(rng() * BUFF_IDS.length)];
    buff = grantBuff(state, id, now);
    pushLog(state, `${buff.icon} ${buff.name}! ${buff.quip}`);
  }

  maybeWorld(state, rng, now);
  const shiny = items.find((i) => RARITY_RANK[i.rarity] >= RARITY_RANK.rare);
  if (shiny) pushLog(state, `${shiny.icon} ${shiny.name} — ${shiny.quip}`);
  else if (items[0] && rng() < 0.18) pushLog(state, `${items[0].icon} ${items[0].quip}`);

  recompute(state);
  const unlocked = checkAchievements(state);
  return { ok: true, items, crit, caps, combo: state.combo, buff, unlocked };
}

function autoScoop(state, rng, now) {
  recompute(state);
  if (state.possumBlocking) return { ok: false, reason: 'possum' };
  const space = state.derived.capacity - state.bag.length;
  if (space <= 0) {
    state.stats.stuffed += 1;
    return { ok: false, reason: 'full' };
  }
  const n = Math.min(space, Math.max(1, Math.ceil(state.derived.pawPower * 0.35)));
  const items = [];
  for (let i = 0; i < n; i += 1) {
    const item = rollItem(state, rng, false);
    state.bag.push(item);
    items.push(item);
    discover(state, item);
    noteRarity(state, item);
  }
  state.stats.items += items.length;
  if (nextLocation(state) && rng() < 0.18) state.bottlecaps += 1;
  maybeWorld(state, rng, now);
  recompute(state);
  return { ok: true, items, auto: true };
}

export function sell(state) {
  recompute(state);
  if (!state.bag.length) return { ok: false, reason: 'empty' };
  const items = state.bag.slice();
  let coins = 0;
  for (const it of items) coins += itemValue(state, it);
  coins = Math.floor(coins);
  state.bag = [];
  state.coins += coins;
  state.lifetimeCoins += coins;
  state.stats.sold += items.length;
  pushLog(state, `Uncle Gary paid ${coins}¢. "I know a seagull."`);
  recompute(state);
  const unlocked = checkAchievements(state);
  return { ok: true, coins, items, unlocked };
}

export function buy(state, kind, id) {
  const list = kind === 'crew' ? CREW : UPGRADES;
  const def = list.find((x) => x.id === id);
  if (!def) return { ok: false, reason: 'missing' };
  const store = kind === 'crew' ? state.crew : state.upgrades;
  const level = store[id] || 0;
  if (isMaxed(def, level)) return { ok: false, reason: 'max' };
  if (!isUnlocked(def, state)) return { ok: false, reason: 'locked' };
  const cost = getCost(def, level);
  if (state.coins < cost) return { ok: false, reason: 'coins' };
  state.coins -= cost;
  store[id] = level + 1;
  recompute(state);
  pushLog(state, `${def.icon} ${def.name} is now level ${store[id]}. Tiny tie: tighter.`);
  const unlocked = checkAchievements(state);
  return { ok: true, cost, level: store[id], unlocked };
}

export function travel(state) {
  const next = nextLocation(state);
  if (!next) return { ok: false, reason: 'end' };
  if (state.bottlecaps < next.capsNeeded) return { ok: false, reason: 'caps' };
  if (state.coins < next.travelCost) return { ok: false, reason: 'coins' };
  state.coins -= next.travelCost;
  state.bottlecaps = 0;
  state.locationIndex += 1;
  state.stats.travels += 1;
  recompute(state);
  pushLog(state, `Bus to ${next.name}. ${next.blurb}`);
  const unlocked = checkAchievements(state);
  return { ok: true, location: next, unlocked };
}

export function prestige(state) {
  if (!canPrestige(state)) return { ok: false, reason: 'locked' };
  const gain = prestigeGain(state);
  const kept = {
    infamy: state.infamy + gain,
    prestigeCount: state.prestigeCount + 1,
    discovered: [...state.discovered],
    achievements: [...state.achievements],
    muted: state.muted,
    stats: { ...createState().stats, ...state.stats, prestiges: (state.stats.prestiges || 0) + 1 },
  };
  const next = createState();
  next.infamy = kept.infamy;
  next.prestigeCount = kept.prestigeCount;
  next.discovered = kept.discovered;
  next.achievements = kept.achievements;
  next.muted = kept.muted;
  next.stats = kept.stats;
  next.started = true;
  for (const key of Object.keys(state)) delete state[key];
  Object.assign(state, next);
  recompute(state);
  pushLog(state, `Incorporated. Infamy x${state.infamy.toFixed(2)}. The dumpster remembers.`);
  checkAchievements(state);
  return { ok: true, infamy: state.infamy, gain };
}

export function petCat(state, now = Date.now()) {
  if (now > state.catUntil) return { ok: false, reason: 'gone' };
  state.catUntil = 0;
  state.stats.pets += 1;
  const buff = grantBuff(state, 'luckstorm', now);
  pushLog(state, 'The alley cat accepted pets as tax. Lucky.');
  const unlocked = checkAchievements(state);
  return { ok: true, buff, unlocked };
}

export function kickPossum(state, now = Date.now()) {
  if (!state.possumBlocking) return { ok: false, reason: 'gone' };
  state.possumBlocking = false;
  state.possumUntil = now + 16000;
  state.stats.kicks += 1;
  pushLog(state, 'You kicked an innocent possum. He will put this on his podcast.');
  const unlocked = checkAchievements(state);
  return { ok: true, unlocked };
}

export function recommend(state) {
  recompute(state);
  if (canTravel(state)) return { kind: 'travel', id: 'travel' };

  const options = [];
  for (const def of UPGRADES) {
    const lv = state.upgrades[def.id] || 0;
    if (!isUnlocked(def, state) || isMaxed(def, lv)) continue;
    const cost = getCost(def, lv);
    if (cost > state.coins) continue;
    let score = 1 / cost;
    if (def.id === 'bag' && state.bag.length >= state.derived.capacity - 2) score *= 8;
    if (def.id === 'paw' && state.derived.pawPower >= state.derived.capacity) score *= 0.2;
    if (def.id === 'speed' && state.derived.scoopMs > 280) score *= 2.2;
    if (def.id === 'value') score *= 1.4;
    options.push({ kind: 'upgrade', id: def.id, score });
  }
  for (const def of CREW) {
    const lv = state.crew[def.id] || 0;
    if (!isUnlocked(def, state) || isMaxed(def, lv)) continue;
    const cost = getCost(def, lv);
    if (cost > state.coins) continue;
    let score = 1.1 / cost;
    if (def.id === 'intern' && lv < 3) score *= 3;
    if (def.id === 'accountant' && (state.crew.intern || 0) >= 1) score *= 4;
    options.push({ kind: 'crew', id: def.id, score });
  }
  options.sort((a, b) => b.score - a.score);
  return options[0] || null;
}

export function tick(state, dt, rng = Math.random, now = Date.now()) {
  state.stats.playMs += dt;
  state.cooldown = Math.max(0, state.cooldown - dt);
  if (state.combo > 0 && now - state.comboAt > COMBO_WINDOW) state.combo = 0;
  state.buffs = state.buffs.filter((b) => b.until > now);
  if (state.possumBlocking && now > state.possumUntil + 4000) {
    // still blocking until kicked; just linger
  }
  if (now > state.catUntil) {
    // cat despawns naturally
  }

  recompute(state);
  const events = [];
  if ((state.crew.intern || 0) > 0) {
    state.internAcc += dt;
    const period = state.derived.internMs;
    let guard = 0;
    while (state.internAcc >= period && period > 0 && guard < 40) {
      guard += 1;
      state.internAcc -= period;
      if (state.possumBlocking) break;
      if (state.bag.length >= state.derived.capacity) {
        if (state.derived.autoSell) events.push({ type: 'autosell', ...sell(state) });
        else break;
      }
      if (state.bag.length < state.derived.capacity) {
        const r = autoScoop(state, rng, now);
        if (r.ok) events.push({ type: 'auto', ...r });
        if (state.derived.autoSell && state.bag.length >= state.derived.capacity) {
          events.push({ type: 'autosell', ...sell(state) });
        }
      }
    }
  }
  return events;
}

export function shopItems(state) {
  recompute(state);
  const rec = recommend(state);
  const map = (list, kind) =>
    list.map((def) => {
      const level = (kind === 'crew' ? state.crew : state.upgrades)[def.id] || 0;
      const unlocked = isUnlocked(def, state);
      const maxed = isMaxed(def, level);
      const cost = maxed ? 0 : getCost(def, level);
      const visible = unlocked || state.lifetimeCoins >= def.baseCost * 0.35;
      return {
        def,
        kind,
        level,
        unlocked,
        maxed,
        cost,
        affordable: unlocked && !maxed && state.coins >= cost,
        visible,
        recommended: rec && rec.kind === kind && rec.id === def.id,
      };
    });
  return {
    gear: map(UPGRADES, 'upgrade'),
    crew: map(CREW, 'crew'),
    rec,
  };
}

export class Game {
  constructor(saved) {
    this.state = hydrate(saved);
    this.listeners = new Set();
  }

  on(fn) {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  emit(type, payload) {
    for (const fn of this.listeners) fn({ type, payload, state: this.state });
  }

  scoop(rng, now) {
    const r = scoop(this.state, rng, now);
    this.emit('scoop', r);
    return r;
  }

  sell() {
    const r = sell(this.state);
    this.emit('sell', r);
    return r;
  }

  buy(kind, id) {
    const r = buy(this.state, kind, id);
    this.emit('buy', r);
    return r;
  }

  travel() {
    const r = travel(this.state);
    this.emit('travel', r);
    return r;
  }

  prestige() {
    const r = prestige(this.state);
    this.emit('prestige', r);
    return r;
  }

  petCat(now) {
    const r = petCat(this.state, now);
    this.emit('pet', r);
    return r;
  }

  kickPossum(now) {
    const r = kickPossum(this.state, now);
    this.emit('kick', r);
    return r;
  }

  tick(dt, rng, now) {
    const events = tick(this.state, dt, rng, now);
    if (events.length) this.emit('tick', events);
    return events;
  }
}
