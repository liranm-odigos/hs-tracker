/** The Scheme — a spend-to-unlock raccoon career tree. */

export const TREE_SIZE = { w: 1120, h: 2680 };

export const TREE_NODES = [
  {
    id: 'root',
    name: 'Tiny Tie',
    icon: '👔',
    branch: 'core',
    x: 550,
    y: 70,
    parents: [],
    max: 1,
    start: 1,
    baseCost: 0,
    desc: 'You showed up. The dumpster is already unionizing.',
    effects: {},
  },

  // ---- row 1: four starters ----
  {
    id: 'paw',
    name: 'Paw Power',
    icon: '🐾',
    branch: 'paws',
    x: 160,
    y: 230,
    parents: ['root'],
    baseCost: 12,
    rate: 1.17,
    desc: 'Grab more junk per scoop. Your paws are becoming a municipal issue.',
    effects: { paw: 1 },
  },
  {
    id: 'bag',
    name: 'Bigger Bag',
    icon: '🎒',
    branch: 'bags',
    x: 390,
    y: 230,
    parents: ['root'],
    baseCost: 16,
    rate: 1.15,
    desc: '+4 pockets. A trash bag with a five-year plan.',
    effects: { bag: 4 },
  },
  {
    id: 'speed',
    name: 'Caffeinated Wrists',
    icon: '☕',
    branch: 'hustle',
    x: 550,
    y: 230,
    parents: ['root'],
    max: 22,
    baseCost: 22,
    rate: 1.2,
    desc: 'Scoop faster. You watched a squirrel do it and took notes.',
    effects: { speed: 1 },
  },
  {
    id: 'value',
    name: 'Shady Appraisal',
    icon: '😏',
    branch: 'nose',
    x: 780,
    y: 230,
    parents: ['root'],
    requiresSold: 1,
    baseCost: 35,
    rate: 1.21,
    desc: '+15% sell value. Uncle Gary taught you to lie with confidence.',
    effects: { value: 0.15 },
  },

  // ---- paws ----
  {
    id: 'crit',
    name: 'Sticky Fingers',
    icon: '✨',
    branch: 'paws',
    x: 160,
    y: 400,
    parents: ['paw'],
    parentMin: { paw: 1 },
    max: 18,
    baseCost: 70,
    rate: 1.23,
    desc: '+2.5% critical scoop. "It fell in my bag."',
    effects: { crit: 0.025 },
  },
  {
    id: 'dual',
    name: 'Dual Paws',
    icon: '🙌',
    branch: 'paws',
    x: 160,
    y: 570,
    parents: ['crit'],
    parentMin: { crit: 1 },
    requiresSpent: 80,
    max: 5,
    baseCost: 220,
    rate: 1.28,
    desc: 'An extra grab per scoop. You have explained this to no one.',
    effects: { extra: 1 },
  },
  {
    id: 'tornado',
    name: 'Trash Tornado',
    icon: '🌪️',
    branch: 'paws',
    x: 160,
    y: 740,
    parents: ['dual'],
    max: 1,
    baseCost: 900,
    desc: '12% chance the scoop happens twice. OSHA has left the chat.',
    effects: { tornado: 0.12 },
  },
  {
    id: 'claws',
    name: 'Municipal Claws',
    icon: '🪝',
    branch: 'paws',
    x: 160,
    y: 910,
    parents: ['tornado'],
    max: 8,
    baseCost: 1400,
    rate: 1.26,
    desc: 'City-grade grabbing. The mayor sent a letter. You ate it.',
    effects: { paw: 1, crit: 0.01 },
  },
  {
    id: 'third',
    name: 'Secret Third Paw',
    icon: '🖐️',
    branch: 'paws',
    x: 160,
    y: 1080,
    parents: ['claws'],
    parentMin: { claws: 2 },
    max: 1,
    baseCost: 8000,
    desc: 'Do not ask where it was. +3 junk per scoop.',
    effects: { paw: 3 },
  },
  {
    id: 'gauntlet',
    name: 'Dumpster Gauntlet',
    icon: '🥊',
    branch: 'paws',
    x: 160,
    y: 1250,
    parents: ['third'],
    max: 1,
    baseCost: 24000,
    desc: 'Crits pull an extra rare. You are a hazard.',
    effects: { critBump: 1 },
  },

  // ---- bags ----
  {
    id: 'pockets',
    name: 'Coat Pockets',
    icon: '🧥',
    branch: 'bags',
    x: 390,
    y: 400,
    parents: ['bag'],
    max: 6,
    baseCost: 90,
    rate: 1.2,
    desc: '+6 capacity. The coat is mostly pockets now.',
    effects: { bag: 6 },
  },
  {
    id: 'vacuum',
    name: 'Vacuum Seal',
    icon: '💨',
    branch: 'bags',
    x: 390,
    y: 570,
    parents: ['pockets'],
    requiresSpent: 120,
    max: 1,
    baseCost: 480,
    desc: 'Overflow sells at 70% instead of sitting on the lid crying.',
    effects: { overflow: 0.7 },
  },
  {
    id: 'dimension',
    name: 'Pocket Dimension',
    icon: '🕳️',
    branch: 'bags',
    x: 390,
    y: 740,
    parents: ['vacuum'],
    max: 1,
    baseCost: 2200,
    desc: '+24 slots. Physics filed a complaint.',
    effects: { bag: 24 },
  },
  {
    id: 'zipper',
    name: 'Cursed Zipper',
    icon: '🤐',
    branch: 'bags',
    x: 390,
    y: 910,
    parents: ['dimension'],
    max: 1,
    baseCost: 3500,
    desc: 'Selling no longer drops your combo. Uncle Gary hates this.',
    effects: { stickyCombo: 1 },
  },
  {
    id: 'bottomless',
    name: 'Bottomless',
    icon: '♾️',
    branch: 'bags',
    x: 390,
    y: 1080,
    parents: ['zipper'],
    max: 1,
    baseCost: 16000,
    desc: '+40 slots. The bag has a weather system.',
    effects: { bag: 40 },
  },

  // ---- hustle ----
  {
    id: 'espresso',
    name: 'Industrial Espresso',
    icon: '🏭',
    branch: 'hustle',
    x: 550,
    y: 400,
    parents: ['speed'],
    parentMin: { speed: 2 },
    max: 8,
    baseCost: 160,
    rate: 1.22,
    desc: 'Even faster wrists. The cup is a bucket.',
    effects: { espresso: 1 },
  },
  {
    id: 'night',
    name: 'Night Shift',
    icon: '🌙',
    branch: 'hustle',
    x: 550,
    y: 570,
    parents: ['espresso'],
    requiresSpent: 60,
    max: 1,
    baseCost: 200,
    desc: 'Interns (once you hire them) scoop 25% faster. You nap professionally.',
    effects: { internHaste: 0.25 },
  },
  {
    id: 'grease',
    name: 'Grease Tolerance',
    icon: '🔥',
    branch: 'hustle',
    x: 550,
    y: 740,
    parents: ['night'],
    max: 5,
    baseCost: 700,
    rate: 1.3,
    desc: 'Power-ups show up more. Your blood is now coffee.',
    effects: { buffLuck: 0.03 },
  },
  {
    id: 'blur',
    name: 'Wrist Blur',
    icon: '⚡',
    branch: 'hustle',
    x: 550,
    y: 910,
    parents: ['grease'],
    parentMin: { grease: 2 },
    max: 1,
    baseCost: 5000,
    desc: 'Minimum scoop time drops to 60ms. Time is a suggestion.',
    effects: { blur: 1 },
  },
  {
    id: 'comboist',
    name: 'Combo Scholar',
    icon: '📚',
    branch: 'hustle',
    x: 550,
    y: 1080,
    parents: ['blur'],
    max: 1,
    baseCost: 9000,
    desc: 'Combo cap 50 and combo is worth more. Hands are a lifestyle.',
    effects: { comboCap: 20, comboValue: 0.02 },
  },

  // ---- nose ----
  {
    id: 'luck',
    name: 'Lucky Nose',
    icon: '👃',
    branch: 'nose',
    x: 780,
    y: 400,
    parents: ['value'],
    baseCost: 110,
    rate: 1.24,
    desc: 'Rarer junk wanders over. You smell tuna and opportunity.',
    effects: { luck: 1 },
  },
  {
    id: 'gourmet',
    name: 'Gourmet',
    icon: '🍽️',
    branch: 'nose',
    x: 780,
    y: 570,
    parents: ['luck'],
    max: 6,
    baseCost: 260,
    rate: 1.25,
    desc: 'You can tell which banana peels are "notes of oak."',
    effects: { value: 0.12, luck: 0.4 },
  },
  {
    id: 'mba',
    name: 'Seagull MBA',
    icon: '🎓',
    branch: 'nose',
    x: 780,
    y: 740,
    parents: ['gourmet'],
    parentMin: { value: 2 },
    max: 8,
    baseCost: 800,
    rate: 1.27,
    desc: '+20% value. The thesis was "garbage." It passed.',
    effects: { value: 0.2 },
  },
  {
    id: 'golden',
    name: 'Golden Lid',
    icon: '🥇',
    branch: 'nose',
    x: 780,
    y: 910,
    parents: ['mba'],
    max: 1,
    baseCost: 4200,
    desc: '+10% crit. The lid winks at you. You wink back.',
    effects: { crit: 0.1 },
  },
  {
    id: 'sniffer',
    name: 'Mythic Sniffer',
    icon: '🦄',
    branch: 'nose',
    x: 780,
    y: 1080,
    parents: ['golden', 'luck'],
    parentMin: { luck: 3 },
    max: 1,
    baseCost: 18000,
    desc: 'Mythics can appear before the moon. The moon is jealous.',
    effects: { mythicEarly: 1, luck: 2 },
  },

  // ---- crew (spend-gated) ----
  {
    id: 'intern',
    name: 'Raccoon Intern',
    icon: '🦝',
    branch: 'crew',
    x: 990,
    y: 400,
    parents: ['root'],
    requiresSpent: 70,
    baseCost: 150,
    rate: 1.28,
    desc: 'Scoops while you plot. Paid in pizza crusts.',
    effects: { intern: 1 },
  },
  {
    id: 'accountant',
    name: 'Uncle Gary Jr.',
    icon: '🎩',
    branch: 'crew',
    x: 990,
    y: 570,
    parents: ['intern'],
    max: 1,
    baseCost: 420,
    desc: 'Auto-sells a full bag. Takes 0% and all of the credit.',
    effects: { autoSell: 1 },
  },
  {
    id: 'lookout',
    name: 'Possum Lookout',
    icon: '👀',
    branch: 'crew',
    x: 990,
    y: 740,
    parents: ['intern'],
    requiresLocation: 1,
    baseCost: 380,
    rate: 1.3,
    desc: 'Better rares. He is a coward, and that is a skill.',
    effects: { lookout: 1 },
  },
  {
    id: 'union',
    name: 'Alley Union',
    icon: '✊',
    branch: 'crew',
    x: 990,
    y: 910,
    parents: ['intern'],
    parentMin: { intern: 2 },
    requiresLocation: 2,
    baseCost: 1400,
    rate: 1.42,
    desc: 'Global x1.12. They demanded a tiny-hat budget.',
    effects: { union: 1 },
  },
  {
    id: 'influencer',
    name: 'Dumpstagram',
    icon: '📱',
    branch: 'crew',
    x: 990,
    y: 1080,
    parents: ['union'],
    max: 6,
    baseCost: 2400,
    rate: 1.32,
    desc: 'Sponsored peels. +18% value. The algorithm is grease.',
    effects: { value: 0.18 },
  },
  {
    id: 'lawyer',
    name: 'Tiny Lawyer',
    icon: '⚖️',
    branch: 'crew',
    x: 990,
    y: 1250,
    parents: ['lookout'],
    parentMin: { lookout: 2 },
    max: 4,
    baseCost: 3200,
    rate: 1.3,
    desc: 'Luck, and a letter that says the dumpster is a yacht.',
    effects: { luck: 1.2 },
  },
  {
    id: 'ceo',
    name: 'Tiny CEO',
    icon: '💼',
    branch: 'crew',
    x: 990,
    y: 1420,
    parents: ['influencer', 'lawyer'],
    max: 1,
    baseCost: 22000,
    desc: 'Global x1.35. The LinkedIn is just a photo of the dumpster.',
    effects: { ceo: 0.35 },
  },

  // ---- mid unique spend gates ----
  {
    id: 'dance',
    name: 'Victory Dance',
    icon: '💃',
    branch: 'core',
    x: 550,
    y: 1250,
    parents: ['root'],
    requiresSpent: 40,
    max: 1,
    baseCost: 80,
    desc: 'Selling makes you dance. This is not optional. This is culture.',
    effects: { dance: 1 },
  },
  {
    id: 'flies',
    name: 'Personal Flies',
    icon: '🪰',
    branch: 'core',
    x: 390,
    y: 1250,
    parents: ['root'],
    requiresSpent: 25,
    max: 1,
    baseCost: 50,
    desc: 'A tiny swarm that believes in you. Pure vibes. +2% value.',
    effects: { value: 0.02, flies: 1 },
  },
  {
    id: 'catcafe',
    name: 'Cat Cafe',
    icon: '🐱',
    branch: 'core',
    x: 720,
    y: 1250,
    parents: ['root'],
    requiresSpent: 150,
    max: 1,
    baseCost: 600,
    desc: 'The alley cat visits more. Pets are a business model.',
    effects: { cat: 1 },
  },
  {
    id: 'possumhr',
    name: 'Possum HR',
    icon: '📋',
    branch: 'core',
    x: 160,
    y: 1420,
    parents: ['lookout'],
    max: 1,
    baseCost: 1100,
    desc: 'Kicking a possum pays 25¢ times your location. He still podcasts.',
    effects: { possumPay: 1 },
  },
  {
    id: 'busking',
    name: 'Bus Pass',
    icon: '🚌',
    branch: 'hustle',
    x: 550,
    y: 1420,
    parents: ['night'],
    max: 5,
    baseCost: 500,
    rate: 1.3,
    desc: 'More bottlecaps. The bus driver is a raccoon in a hat.',
    effects: { caps: 0.08 },
  },
  {
    id: 'magnetperm',
    name: 'Dumpster Magnet',
    icon: '🧲',
    branch: 'nose',
    x: 780,
    y: 1420,
    parents: ['gourmet'],
    max: 1,
    baseCost: 2800,
    desc: 'Permanent +1 junk per scoop. Trash slides toward you like rent.',
    effects: { extra: 1 },
  },

  // ---- empire (late, spend a lot) ----
  {
    id: 'franchise',
    name: 'Franchise Papers',
    icon: '📄',
    branch: 'empire',
    x: 390,
    y: 1620,
    parents: ['ceo'],
    requiresSpent: 800,
    max: 4,
    baseCost: 12000,
    rate: 1.4,
    desc: 'Every dumpster pays +25%. Capitalism, but tiny.',
    effects: { franchise: 0.25 },
  },
  {
    id: 'lobbyist',
    name: 'Lobbyist',
    icon: '🏦',
    branch: 'empire',
    x: 550,
    y: 1620,
    parents: ['franchise'],
    max: 1,
    baseCost: 18000,
    desc: 'Bus tickets cost 30% less. You "know a guy" at city hall.',
    effects: { cheapTravel: 0.3 },
  },
  {
    id: 'second',
    name: 'Second Dumpster',
    icon: '🗑️',
    branch: 'empire',
    x: 720,
    y: 1620,
    parents: ['franchise'],
    max: 1,
    baseCost: 26000,
    desc: 'You scoop two lids. +2 junk. The intern is not paid extra.',
    effects: { extra: 2 },
  },
  {
    id: 'hats',
    name: 'Tiny Hat Budget',
    icon: '🎩',
    branch: 'empire',
    x: 990,
    y: 1620,
    parents: ['union'],
    requiresSpent: 500,
    max: 6,
    baseCost: 4000,
    rate: 1.35,
    desc: 'Union multiplier ticks extra. The hats have a union too.',
    effects: { union: 1 },
  },
  {
    id: 'moonpass',
    name: 'Orbit Permit',
    icon: '🚀',
    branch: 'empire',
    x: 550,
    y: 1810,
    parents: ['lobbyist', 'second'],
    max: 1,
    baseCost: 80000,
    desc: 'Last dumpsters are cheaper to reach. NASA is not amused.',
    effects: { cheapTravel: 0.4 },
  },
  {
    id: 'meatball',
    name: 'Meatball Friend',
    icon: '🧆',
    branch: 'empire',
    x: 160,
    y: 1810,
    parents: ['sniffer'],
    max: 1,
    baseCost: 14000,
    desc: 'It winks during scoops. +luck. You winking back is the tax.',
    effects: { luck: 2 },
  },
  {
    id: 'intern2',
    name: 'Intern Union',
    icon: '📣',
    branch: 'crew',
    x: 990,
    y: 1810,
    parents: ['intern', 'union'],
    max: 8,
    baseCost: 3500,
    rate: 1.3,
    desc: 'Each level is another intern who brought a tiny clipboard.',
    effects: { intern: 1 },
  },
  {
    id: 'legend',
    name: 'Living Legend',
    icon: '🌟',
    branch: 'empire',
    x: 550,
    y: 2000,
    parents: ['moonpass', 'gauntlet', 'bottomless'],
    max: 1,
    baseCost: 250000,
    desc: 'Global x2. Statues of you are also dumpsters.',
    effects: { ceo: 1 },
  },
  {
    id: 'god',
    name: 'Dumpster God',
    icon: '🪐',
    branch: 'empire',
    x: 550,
    y: 2200,
    parents: ['legend'],
    requiresSpent: 20000,
    max: 1,
    baseCost: 1e6,
    desc: 'The dumpster scoops you. +5 junk, +50% value, flies with jobs.',
    effects: { paw: 5, value: 0.5, flies: 1 },
  },
  {
    id: 'silk',
    name: 'Silk Tie',
    icon: '✨',
    branch: 'core',
    x: 780,
    y: 2000,
    parents: ['ceo', 'dance'],
    max: 1,
    baseCost: 40000,
    desc: 'Prestige infamy gain +25%. The tie has a LinkedIn.',
    effects: { infamyGain: 0.25 },
  },
];

export const TREE_BY_ID = Object.fromEntries(TREE_NODES.map((n) => [n.id, n]));

export function nodeLevel(state, id) {
  return state.tree?.[id] || 0;
}

export function parentsMet(state, node) {
  if (!node.parents?.length) return true;
  return node.parents.every((pid) => nodeLevel(state, pid) >= (node.parentMin?.[pid] || 1));
}

export function nodeUnlocked(state, node) {
  if (node.start) return true;
  if (!parentsMet(state, node)) return false;
  if (node.requiresSpent && (state.spent || 0) < node.requiresSpent) return false;
  if (node.requiresSold && (state.stats?.sold || 0) < node.requiresSold) return false;
  if (node.requiresLocation && (state.locationIndex || 0) < node.requiresLocation) return false;
  return true;
}

/** Next-ring nodes are visible once a parent is owned, so the tree grows as you spend. */
export function nodeVisible(state, node) {
  if (node.start || nodeUnlocked(state, node)) return true;
  if (node.parents?.some((pid) => nodeLevel(state, pid) > 0 || TREE_BY_ID[pid]?.start)) return true;
  return false;
}

export function nodeCost(node, level) {
  if (node.start) return 0;
  const rate = node.rate ?? 1.17;
  return Math.floor((node.baseCost || 0) * rate ** level * (1 + 0.035 * level));
}

export function nodeMaxed(node, level) {
  return Boolean(node.max) && level >= node.max;
}

export function sumEffects(state) {
  const fx = {
    paw: 0,
    bag: 0,
    speed: 0,
    espresso: 0,
    value: 0,
    crit: 0,
    luck: 0,
    intern: 0,
    lookout: 0,
    union: 0,
    extra: 0,
    autoSell: 0,
    overflow: 0,
    stickyCombo: 0,
    internHaste: 0,
    buffLuck: 0,
    blur: 0,
    comboCap: 0,
    comboValue: 0,
    mythicEarly: 0,
    ceo: 0,
    dance: 0,
    flies: 0,
    cat: 0,
    possumPay: 0,
    caps: 0,
    franchise: 0,
    cheapTravel: 0,
    infamyGain: 0,
    tornado: 0,
    critBump: 0,
  };
  for (const node of TREE_NODES) {
    const lv = nodeLevel(state, node.id);
    if (!lv || !node.effects) continue;
    for (const [k, v] of Object.entries(node.effects)) {
      fx[k] = (fx[k] || 0) + v * lv;
    }
  }
  return fx;
}

export function migrateTree(state) {
  if (!state.tree || typeof state.tree !== 'object') state.tree = {};
  if (!state.tree.root) state.tree.root = 1;
  if (state.spent == null) state.spent = 0;
  const map = [
    ['paw', 'upgrades'],
    ['bag', 'upgrades'],
    ['speed', 'upgrades'],
    ['value', 'upgrades'],
    ['crit', 'upgrades'],
    ['luck', 'upgrades'],
    ['intern', 'crew'],
    ['accountant', 'crew'],
    ['lookout', 'crew'],
    ['union', 'crew'],
  ];
  for (const [id, store] of map) {
    const legacy = state[store]?.[id] || 0;
    if (legacy && !state.tree[id]) state.tree[id] = legacy;
  }
  return state;
}

export function syncLegacy(state) {
  const fx = sumEffects(state);
  state.upgrades = {
    ...state.upgrades,
    paw: nodeLevel(state, 'paw'),
    bag: nodeLevel(state, 'bag'),
    speed: nodeLevel(state, 'speed'),
    value: nodeLevel(state, 'value'),
    crit: nodeLevel(state, 'crit'),
    luck: nodeLevel(state, 'luck'),
  };
  state.crew = {
    ...state.crew,
    intern: fx.intern,
    accountant: fx.autoSell ? 1 : 0,
    lookout: fx.lookout,
    union: fx.union,
  };
}

export function treeView(state) {
  return TREE_NODES.map((node) => {
    const level = nodeLevel(state, node.id);
    const unlocked = nodeUnlocked(state, node);
    const visible = nodeVisible(state, node);
    const maxed = nodeMaxed(node, level);
    const cost = maxed || node.start ? 0 : nodeCost(node, level);
    return {
      node,
      level,
      unlocked,
      visible,
      maxed,
      cost,
      affordable: unlocked && !maxed && !node.start && state.coins >= cost,
    };
  });
}

export function nextUnlockHint(state, node) {
  if (nodeUnlocked(state, node)) return '';
  const missing = [];
  for (const pid of node.parents || []) {
    const need = node.parentMin?.[pid] || 1;
    const have = nodeLevel(state, pid);
    if (have < need) missing.push(`${TREE_BY_ID[pid]?.name || pid} Lv ${need}`);
  }
  if (node.requiresSpent && (state.spent || 0) < node.requiresSpent) {
    missing.push(`spend ${node.requiresSpent}¢ on the tree`);
  }
  if (node.requiresSold && (state.stats?.sold || 0) < node.requiresSold) missing.push('sell a bag');
  if (node.requiresLocation && (state.locationIndex || 0) < node.requiresLocation) {
    missing.push('a better dumpster');
  }
  return missing.length ? 'Needs: ' + missing.join(', ') : 'Locked';
}

export function affordableTreeCount(state) {
  return treeView(state).filter((x) => x.affordable).length;
}
