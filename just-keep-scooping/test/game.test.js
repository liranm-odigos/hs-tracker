import test from 'node:test';
import assert from 'node:assert/strict';
import {
  buy,
  canTravel,
  createState,
  getCost,
  hydrate,
  prestige,
  rarityWeights,
  recompute,
  rollItem,
  scoop,
  sell,
  tick,
  travel,
} from '../src/game.js';
import { formatNumber } from '../src/format.js';
import { LOCATIONS, UPGRADES } from '../src/data.js';
import { TREE_NODES, nodeUnlocked } from '../src/tree.js';

function rngSeq(values) {
  let i = 0;
  return () => values[i++ % values.length];
}

test('formatNumber uses suffixes', () => {
  assert.equal(formatNumber(12), '12');
  assert.equal(formatNumber(1500), '1.5K');
  assert.equal(formatNumber(2_000_000), '2M');
});

test('a scoop fills the bag and a sell pays coins', () => {
  const s = createState();
  const r = scoop(s, rngSeq([0.01, 0.5, 0.5, 0.5]), 1_000);
  assert.equal(r.ok, true);
  assert.ok(r.items.length >= 1);
  assert.equal(s.bag.length, r.items.length);
  const sold = sell(s);
  assert.equal(sold.ok, true);
  assert.ok(sold.coins >= 1);
  assert.equal(s.bag.length, 0);
  assert.equal(s.coins, sold.coins);
});

test('full bag refuses to scoop', () => {
  const s = createState();
  s.bag = Array.from({ length: 8 }, () => ({
    id: 'peel',
    name: 'Banana Peel',
    rarity: 'common',
    value: 1,
    icon: '🍌',
    quip: 'x',
  }));
  recompute(s);
  const r = scoop(s, () => 0.1, 2_000);
  assert.equal(r.ok, false);
  assert.equal(r.reason, 'full');
});

test('upgrade costs scale and paw power rises', () => {
  const def = UPGRADES.find((u) => u.id === 'paw');
  assert.ok(getCost(def, 1) > getCost(def, 0));
  const s = createState();
  s.coins = 1000;
  const r = buy(s, 'upgrade', 'paw');
  assert.equal(r.ok, true);
  assert.equal(s.upgrades.paw, 1);
  assert.equal(s.derived.pawPower, 2);
});

test('cannot buy what you cannot afford', () => {
  const s = createState();
  s.coins = 0;
  const r = buy(s, 'upgrade', 'paw');
  assert.equal(r.ok, false);
  assert.equal(r.reason, 'coins');
});

test('travel wants bottlecaps and a ticket', () => {
  const s = createState();
  assert.equal(canTravel(s), false);
  const next = LOCATIONS[1];
  s.bottlecaps = next.capsNeeded;
  s.coins = next.travelCost;
  assert.equal(canTravel(s), true);
  const r = travel(s);
  assert.equal(r.ok, true);
  assert.equal(s.locationIndex, 1);
  assert.equal(s.bottlecaps, 0);
});

test('interns scoop and accountants sell', () => {
  const s = createState();
  s.crew.intern = 3;
  s.crew.accountant = 1;
  recompute(s);
  const events = tick(s, 20_000, rngSeq([0.2, 0.3, 0.4]), 50_000);
  assert.ok(events.length > 0);
  assert.ok(s.stats.items > 0 || s.stats.sold > 0 || s.coins > 0);
});

test('prestige keeps infamy and wipes the bag', () => {
  const s = createState();
  s.locationIndex = 5;
  s.lifetimeCoins = 90_000;
  s.coins = 500;
  s.bag.push({ id: 'peel', name: 'Banana Peel', rarity: 'common', value: 1, icon: '🍌', quip: 'x' });
  s.discovered = ['peel'];
  const before = s.infamy;
  const r = prestige(s);
  assert.equal(r.ok, true);
  assert.ok(s.infamy > before);
  assert.equal(s.bag.length, 0);
  assert.equal(s.coins, 0);
  assert.deepEqual(s.discovered, ['peel']);
});

test('hydrate restores a save without exploding', () => {
  const s = hydrate({ coins: 12, upgrades: { paw: 2 }, locationIndex: 0 });
  assert.equal(s.coins, 12);
  assert.equal(s.derived.pawPower, 3);
});

test('later dumpsters make rare junk more likely', () => {
  const early = createState();
  const late = createState();
  late.locationIndex = 8;
  late.upgrades.luck = 8;
  const a = rarityWeights(early);
  const b = rarityWeights(late);
  assert.ok(b.common < a.common);
  assert.ok(b.legendary > a.legendary);
});

test('rollItem always returns a real piece of junk', () => {
  const s = createState();
  for (let i = 0; i < 30; i += 1) {
    const item = rollItem(s, Math.random);
    assert.ok(item.id);
    assert.ok(item.value >= 1);
  }
});

test('the scheme is a big branching tree', () => {
  assert.ok(TREE_NODES.length >= 40);
  const withParents = TREE_NODES.filter((n) => n.parents?.length);
  assert.ok(withParents.length >= 30);
});

test('spending on the tree unlocks intern raccoons', () => {
  const s = createState();
  s.coins = 8000;
  s.stats.sold = 1;
  const intern = TREE_NODES.find((n) => n.id === 'intern');
  assert.equal(nodeUnlocked(s, intern), false);
  let guard = 0;
  while ((s.spent || 0) < 70 && guard < 40) {
    guard += 1;
    const r = buy(s, 'tree', 'paw');
    assert.equal(r.ok, true);
  }
  assert.equal(nodeUnlocked(s, intern), true);
  const hired = buy(s, 'tree', 'intern');
  assert.equal(hired.ok, true);
  assert.ok(s.derived.intern >= 1);
});
