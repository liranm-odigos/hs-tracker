import { ACHIEVEMENTS, BUFFS, LOCATIONS, NEWS, TRASH } from './data.js';
import { formatCoins } from './format.js';
import {
  buySound,
  coinSound,
  denySound,
  kickSound,
  petSound,
  scoopSound,
  setMuted,
  travelSound,
  unlockAudio,
} from './audio.js';
import {
  canPrestige,
  canTravel,
  capsProgress,
  nextLocation,
  prestigeGain,
  shopItems,
} from './game.js';

const RACCOON_SVG = `
<svg viewBox="0 0 160 160" aria-hidden="true">
  <ellipse cx="80" cy="138" rx="46" ry="12" fill="#000" opacity=".25"/>
  <ellipse cx="80" cy="108" rx="42" ry="36" fill="#6b5848"/>
  <ellipse cx="80" cy="118" rx="28" ry="16" fill="#cbb7a0"/>
  <rect x="74" y="112" width="12" height="26" rx="3" fill="#2a1c10"/>
  <rect x="72" y="112" width="16" height="8" rx="2" fill="#ffd166"/>
  <circle cx="80" cy="62" r="32" fill="#6b5848"/>
  <ellipse cx="56" cy="40" rx="14" ry="16" fill="#6b5848"/>
  <ellipse cx="104" cy="40" rx="14" ry="16" fill="#6b5848"/>
  <ellipse cx="56" cy="40" rx="8" ry="10" fill="#2a2430"/>
  <ellipse cx="104" cy="40" rx="8" ry="10" fill="#2a2430"/>
  <ellipse cx="64" cy="62" rx="16" ry="12" fill="#2a2430"/>
  <ellipse cx="96" cy="62" rx="16" ry="12" fill="#2a2430"/>
  <circle cx="68" cy="62" r="5" fill="#f7f1e8"/>
  <circle cx="92" cy="62" r="5" fill="#f7f1e8"/>
  <circle cx="69" cy="63" r="2.2" fill="#1a1224"/>
  <circle cx="93" cy="63" r="2.2" fill="#1a1224"/>
  <ellipse cx="80" cy="74" rx="6" ry="4" fill="#1a1224"/>
  <path d="M74 80 Q80 86 86 80" fill="none" stroke="#1a1224" stroke-width="2"/>
  <ellipse cx="48" cy="108" rx="14" ry="10" fill="#6b5848"/>
  <ellipse cx="112" cy="108" rx="14" ry="10" fill="#6b5848"/>
  <ellipse cx="48" cy="108" rx="8" ry="6" fill="#cbb7a0"/>
  <ellipse cx="112" cy="108" rx="8" ry="6" fill="#cbb7a0"/>
  <ellipse cx="70" cy="140" rx="10" ry="7" fill="#2a2430"/>
  <ellipse cx="94" cy="140" rx="10" ry="7" fill="#2a2430"/>
</svg>`;

function el(html) {
  const t = document.createElement('template');
  t.innerHTML = html.trim();
  return t.content.firstElementChild;
}

export function mount(root, game) {
  root.innerHTML = `
    <div class="grain"></div>
    <div class="shell">
      <header class="top">
        <div class="brand">
          <div class="logo">Just Keep Scooping</div>
          <div class="tag">garbage · capitalism · tiny tie</div>
        </div>
        <div class="hud">
          <div class="chip" data-hud="coins">💰 <b>0¢</b></div>
          <div class="chip bag" data-hud="bag">🎒 0/8</div>
          <div class="chip" data-hud="caps">🧢 0</div>
          <div class="chip" data-hud="infamy">😈 x1</div>
        </div>
      </header>
      <div class="ticker"><div class="ticker-track">${NEWS.concat(NEWS).map((n) => `<span>🦝 ${n}</span>`).join('')}</div></div>
      <div class="layout">
        <section class="stage sky-diner" data-stage>
          <div class="stars"></div>
          <div class="moon"></div>
          <div class="city"></div>
          <div class="ground"></div>
          <div class="neon" data-neon>EAT HERE OR ELSE</div>
          <div class="combo" data-combo>x1 COMBO</div>
          <div class="buffs" data-buffs></div>
          <div class="raccoon" data-raccoon>${RACCOON_SVG}</div>
          <div class="dumpster" data-dumpster>
            <div class="lid"><div class="lid-handle"></div></div>
            <div class="dump-body"></div>
          </div>
          <div class="gary" data-gary title="Sell to Uncle Gary">
            <div class="gary-art">🦝</div>
            <div class="gary-label">UNCLE GARY</div>
          </div>
          <div class="cat" data-cat hidden>🐱</div>
          <div class="possum" data-possum hidden>😾</div>
          <div class="fx" data-fx></div>
          <div class="bag-meter" data-meter><span></span></div>
          <div class="stage-actions">
            <button class="btn scoop" data-scoop>SCOOP</button>
            <button class="btn sell" data-sell disabled>SELL</button>
          </div>
        </section>
        <aside class="side" data-side></aside>
      </div>
    </div>
    <div class="overlay" data-title>
      <div class="title-card">
        <div style="font-size:64px">🦝</div>
        <h1>JUST KEEP SCOOPING</h1>
        <p>A tiny incremental about turning dumpsters into a criminal empire.</p>
        <p>You are Scoop. You have paws. Uncle Gary has coins. The rest is capitalism.</p>
        <button class="btn scoop" data-start>I HAVE PAWS</button>
        <div class="help">Space to scoop · S to sell · M to mute</div>
        <div class="footer-btns" style="justify-content:center;margin-top:16px">
          <button class="linkish" data-continue hidden>Keep scooping</button>
        </div>
      </div>
    </div>
    <div class="overlay hidden" data-modal-wrap>
      <div class="modal" data-modal></div>
    </div>
    <div class="toasts" data-toasts></div>
  `;

  const $ = (sel) => root.querySelector(sel);
  const stage = $('[data-stage]');
  const raccoon = $('[data-raccoon]');
  const dumpster = $('[data-dumpster]');
  const fx = $('[data-fx]');
  const side = $('[data-side]');
  const toasts = $('[data-toasts]');
  const title = $('[data-title]');
  const modalWrap = $('[data-modal-wrap]');
  const modal = $('[data-modal]');
  const startBtn = $('[data-start]');
  const continueBtn = $('[data-continue]');

  let shownCoins = game.state.coins;
  let scoopLock = false;

  if (game.state.started && (game.state.stats.scoops > 0 || game.state.coins > 0)) {
    continueBtn.hidden = false;
    startBtn.textContent = 'NEW GAME (wipe save)';
  }

  function toast(titleText, desc) {
    const node = el(`<div class="toast"><b>${titleText}</b>${desc ? `<small>${desc}</small>` : ''}</div>`);
    toasts.append(node);
    setTimeout(() => node.remove(), 3200);
  }

  function spawnFx(html, x, y) {
    const node = el(html);
    node.style.left = x + 'px';
    node.style.top = y + 'px';
    fx.append(node);
    setTimeout(() => node.remove(), 900);
  }

  function juiceScoop(result) {
    raccoon.classList.remove('scooping');
    void raccoon.offsetWidth;
    raccoon.classList.add('scooping');
    dumpster.classList.add('open');
    setTimeout(() => dumpster.classList.remove('open'), 220);
    const box = dumpster.getBoundingClientRect();
    const stageBox = stage.getBoundingClientRect();
    const x = box.left - stageBox.left + box.width * 0.45;
    const y = box.top - stageBox.top + 40;
    for (const item of result.items || []) {
      spawnFx(`<div class="floater">${item.icon}</div>`, x + Math.random() * 40, y);
    }
    if (result.crit) {
      spawnFx(`<div class="popnum r-legendary">CRIT!</div>`, x, y - 20);
      stage.classList.add('shake');
      setTimeout(() => stage.classList.remove('shake'), 350);
    }
    const rare = (result.items || []).find((i) => ['rare', 'epic', 'legendary', 'mythic'].includes(i.rarity));
    if (rare) {
      spawnFx(`<div class="splash r-${rare.rarity}">${rare.icon} ${rare.name}!</div>`, stage.clientWidth / 2, 80);
    }
    const top = result.items?.[0];
    scoopSound(Boolean(result.crit), top?.rarity || 'common');
    for (const a of result.unlocked || []) toast('🏆 ' + a.name, a.desc);
  }

  function doScoop() {
    if (scoopLock) return;
    const r = game.scoop();
    if (!r.ok) {
      if (r.reason === 'cooldown') return;
      denySound();
      if (r.reason === 'full') {
        raccoon.classList.add('sad');
        $('[data-scoop]').classList.add('pulse');
        $('[data-sell]').classList.add('pulse');
        toast('Bag full', 'Sell to Uncle Gary before the health inspector arrives.');
      }
      if (r.reason === 'possum') toast('Possum on the lid', 'Kick that professional napper.');
      return;
    }
    raccoon.classList.remove('sad');
    scoopLock = true;
    juiceScoop(r);
    setTimeout(() => {
      scoopLock = false;
    }, Math.min(220, game.state.derived.scoopMs));
  }

  function doSell() {
    const r = game.sell();
    if (!r.ok) {
      denySound();
      return;
    }
    coinSound();
    $('[data-gary]').classList.remove('pay');
    void $('[data-gary]').offsetWidth;
    $('[data-gary]').classList.add('pay');
    raccoon.classList.remove('sad');
    $('[data-scoop]').classList.remove('pulse');
    $('[data-sell]').classList.remove('pulse');
    const box = $('[data-gary]').getBoundingClientRect();
    const stageBox = stage.getBoundingClientRect();
    spawnFx(
      `<div class="popnum r-legendary">+${formatCoins(r.coins)}</div>`,
      box.left - stageBox.left + 40,
      box.top - stageBox.top,
    );
    for (const a of r.unlocked || []) toast('🏆 ' + a.name, a.desc);
  }

  let lastSideKey = '';
  function sideKey(s) {
    return [
      Math.floor(s.coins),
      s.bag.length,
      s.bottlecaps,
      s.locationIndex,
      JSON.stringify(s.upgrades),
      JSON.stringify(s.crew),
      s.log[0] || '',
      s.muted,
      s.achievements.length,
      s.discovered.length,
      s.prestigeCount,
      canTravel(s) ? 1 : 0,
    ].join('|');
  }
  function renderSide(force = false) {
    const s = game.state;
    const key = sideKey(s);
    if (!force && key === lastSideKey) return;
    lastSideKey = key;
    const shop = shopItems(s);
    const next = nextLocation(s);
    const loc = LOCATIONS[s.locationIndex];
    const gear = shop.gear
      .filter((x) => x.visible)
      .map((x) => shopCard(x))
      .join('');
    const crew = shop.crew
      .filter((x) => x.visible)
      .map((x) => shopCard(x))
      .join('');

    let travel = '';
    if (next) {
      const ready = canTravel(s);
      travel = `
        <div class="travel">
          <h3>🚌 Next dumpster: ${next.name}</h3>
          <p>${next.blurb}</p>
          <div class="bar"><span style="width:${Math.round(capsProgress(s) * 100)}%"></span></div>
          <p>Bottlecaps ${Math.min(s.bottlecaps, next.capsNeeded)}/${next.capsNeeded} · ticket ${formatCoins(next.travelCost)}</p>
          <button class="btn ${ready ? 'pulse' : ''}" data-travel ${ready ? '' : 'disabled'}>TAKE THE BUS</button>
        </div>`;
    } else {
      travel = `<div class="travel"><h3>🌙 End of the line</h3><p>${loc.blurb} You scooped the horizon. Incorporate for more infamy.</p></div>`;
    }

    let prestige = '';
    if (canPrestige(s) || s.prestigeCount > 0) {
      const g = prestigeGain(s);
      prestige = `
        <div class="prestige">
          <h3>🏢 Incorporate</h3>
          <p>Reset the empire. Keep discoveries, achievements, and +${g.toFixed(2)} infamy. Numbers go even more up.</p>
          <button class="btn" data-prestige ${canPrestige(s) ? '' : 'disabled'}>PRESTIGE  x${(s.infamy + g).toFixed(2)}</button>
        </div>`;
    }

    side.innerHTML = `
      <h2>${loc.name}</h2>
      <p class="section-label">${loc.blurb} · x${loc.mult} dumpster</p>
      <div class="section-label">Always buy more paws</div>
      ${gear}
      ${travel}
      <div class="section-label">Crew</div>
      ${crew || '<p class="help">Earn a little, then hire raccoons.</p>'}
      ${prestige}
      <div class="section-label">Uncle Gary says</div>
      <div class="log">${(s.log.length ? s.log : ['The dumpster hums. It knows.']).map((l) => `<div>› ${l}</div>`).join('')}</div>
      <div class="footer-btns">
        <button class="linkish" data-open="codex">Codex ${s.discovered.length}/${TRASH.length}</button>
        <button class="linkish" data-open="ach">Trophies ${s.achievements.length}/${ACHIEVEMENTS.length}</button>
        <button class="linkish" data-mute>${s.muted ? 'Sound off' : 'Sound on'}</button>
        <button class="linkish" data-open="help">How to play</button>
      </div>
    `;
  }

  function shopCard(x) {
    const { def, level, unlocked, maxed, cost, affordable, recommended, kind } = x;
    const cls = [
      'card',
      affordable ? 'affordable' : '',
      recommended ? 'recommended' : '',
      !unlocked ? 'locked' : '',
      maxed ? 'maxed' : '',
    ].join(' ');
    const hint = !unlocked ? def.unlockHint : maxed ? 'MAX' : def.next(level);
    const label = maxed ? 'MAX' : formatCoins(cost);
    return `
      <div class="${cls}">
        <div class="ico">${def.icon}</div>
        <div>
          <h3>${def.name} <span style="color:var(--muted)">Lv ${level}</span>${recommended ? ' · HOT' : ''}</h3>
          <p>${unlocked ? def.desc : 'Locked: ' + def.unlockHint}</p>
          <p>${hint} · ${def.current(level)}</p>
        </div>
        <button class="buy ${affordable ? 'ready' : ''}" data-buy="${kind}:${def.id}" ${unlocked && !maxed ? '' : 'disabled'}>${label}</button>
      </div>`;
  }

  function renderHud(dt) {
    const s = game.state;
    const speed = Math.max(8, Math.abs(s.coins - shownCoins) * 8);
    if (shownCoins < s.coins) shownCoins = Math.min(s.coins, shownCoins + speed * (dt / 1000));
    else shownCoins = s.coins;
    $('[data-hud="coins"]').innerHTML = `💰 <b>${formatCoins(Math.floor(shownCoins))}</b>`;
    const cap = s.derived.capacity;
    const bagChip = $('[data-hud="bag"]');
    bagChip.textContent = `🎒 ${s.bag.length}/${cap}`;
    bagChip.classList.toggle('full', s.bag.length >= cap);
    $('[data-hud="caps"]').textContent = `🧢 ${s.bottlecaps}`;
    $('[data-hud="infamy"]').textContent = `😈 x${s.infamy.toFixed(2)}`;
    $('[data-neon]').textContent = s.derived.location.neon;
    for (const cls of [...stage.classList]) {
      if (cls.startsWith('sky-')) stage.classList.remove(cls);
    }
    stage.classList.add('sky-' + s.derived.location.sky);
    const combo = $('[data-combo]');
    combo.textContent = `x${s.combo} COMBO`;
    combo.classList.toggle('on', s.combo >= 2);
    const meter = $('[data-meter]');
    meter.classList.toggle('full', s.bag.length >= cap);
    meter.firstElementChild.style.width = `${Math.min(100, (s.bag.length / cap) * 100)}%`;
    $('[data-sell]').disabled = s.bag.length === 0;
    $('[data-scoop]').textContent = s.possumBlocking ? 'KICK?!' : s.bag.length >= cap ? 'BAG FULL' : 'SCOOP';
    $('[data-cat]').hidden = Date.now() > s.catUntil;
    $('[data-possum]').hidden = !s.possumBlocking;
    raccoon.classList.toggle('sad', s.bag.length >= cap);
    const buffHtml = s.buffs
      .map((b) => {
        const left = Math.max(0, b.until - Date.now());
        const def = BUFFS[b.id];
        return `<div class="buff">${def?.icon ?? ''} ${def?.name ?? b.id} ${Math.ceil(left / 1000)}s</div>`;
      })
      .join('');
    $('[data-buffs]').innerHTML = buffHtml;
  }

  function openModal(kind) {
    const s = game.state;
    if (kind === 'codex') {
      modal.innerHTML = `<h2>Codex of Junk</h2><p>If it was in a dumpster, it is culture.</p><div class="codex-grid">${TRASH.map((t) => {
        const known = s.discovered.includes(t.id);
        return `<div class="codex-item"><div class="big">${known ? t.icon : '❓'}</div><div class="r-${t.rarity}">${known ? t.name : '???'}</div><div>${known ? t.quip : 'Not scooped yet.'}</div></div>`;
      }).join('')}</div><div class="footer-btns"><button class="btn" data-close>Close</button></div>`;
    } else if (kind === 'ach') {
      modal.innerHTML = `<h2>Tiny Trophies</h2>${ACHIEVEMENTS.map((a) => {
        const got = s.achievements.includes(a.id);
        return `<div class="card ${got ? 'affordable' : 'locked'}"><div class="ico">${got ? '🏆' : '🔒'}</div><div><h3>${a.name}</h3><p>${a.desc}</p></div></div>`;
      }).join('')}<div class="footer-btns"><button class="btn" data-close>Close</button></div>`;
    } else {
      modal.innerHTML = `<h2>How to play</h2>
        <p>Scoop the dumpster. Fill the bag. Sell to Uncle Gary. Buy anything that makes numbers go up.</p>
        <p>Bottlecaps (they don't take bag space) buy the next dumpster. Crew scoops while you plot. At the end of the line, incorporate for infamy.</p>
        <p>Pet the cat. Kick the possum. This is legally a workplace.</p>
        <p><b>Space</b> scoop · <b>S</b> sell · <b>M</b> mute</p>
        <div class="footer-btns"><button class="btn" data-close>Close</button></div>`;
    }
    modalWrap.classList.remove('hidden');
  }

  startBtn.addEventListener('click', () => {
    unlockAudio();
    if (game.state.started && (game.state.stats.scoops > 0 || game.state.coins > 0) && startBtn.textContent.includes('NEW')) {
      if (confirm('Wipe the empire and start as a nobody raccoon?')) {
        game.state.started = true;
        root.dispatchEvent(new CustomEvent('new-game', { bubbles: true }));
      }
      return;
    }
    game.state.started = true;
    title.classList.add('hidden');
  });
  continueBtn.addEventListener('click', () => {
    unlockAudio();
    game.state.started = true;
    title.classList.add('hidden');
  });

  $('[data-scoop]').addEventListener('click', () => {
    if (!game.state.started) return;
    if (game.state.possumBlocking) {
      const r = game.kickPossum();
      if (r.ok) {
        kickSound();
        toast('Workplace drama', 'The possum is writing a 1-star review.');
        for (const a of r.unlocked || []) toast('🏆 ' + a.name, a.desc);
      }
      return;
    }
    doScoop();
  });
  $('[data-sell]').addEventListener('click', doSell);
  $('[data-gary]').addEventListener('click', doSell);
  $('[data-cat]').addEventListener('click', () => {
    const r = game.petCat();
    if (r.ok) {
      petSound();
      toast('Purr tax paid', 'Lucky storm incoming.');
    }
  });

  side.addEventListener('click', (e) => {
    const buyBtn = e.target.closest('[data-buy]');
    if (buyBtn) {
      const [kind, id] = buyBtn.getAttribute('data-buy').split(':');
      const r = game.buy(kind, id);
      if (r.ok) {
        buySound();
        toast('Upgrade acquired', 'The tiny tie tightens.');
        for (const a of r.unlocked || []) toast('🏆 ' + a.name, a.desc);
      } else denySound();
      renderSide();
      return;
    }
    if (e.target.closest('[data-travel]')) {
      const r = game.travel();
      if (r.ok) {
        travelSound();
        toast('New dumpster', r.location.name);
        stage.classList.add('shake');
        setTimeout(() => stage.classList.remove('shake'), 350);
      } else denySound();
      renderSide();
      return;
    }
    if (e.target.closest('[data-prestige]')) {
      if (!confirm('Incorporate? You keep infamy, trophies, and the codex. Everything else resets.')) return;
      const r = game.prestige();
      if (r.ok) {
        travelSound();
        toast('Incorporated', `Infamy x${r.infamy.toFixed(2)}`);
      }
      renderSide();
      return;
    }
    const open = e.target.closest('[data-open]');
    if (open) openModal(open.getAttribute('data-open'));
    if (e.target.closest('[data-mute]')) {
      game.state.muted = !game.state.muted;
      setMuted(game.state.muted);
      renderSide();
    }
  });

  modalWrap.addEventListener('click', (e) => {
    if (e.target === modalWrap || e.target.closest('[data-close]')) modalWrap.classList.add('hidden');
  });

  window.addEventListener('keydown', (e) => {
    if (!game.state.started || title.classList.contains('hidden') === false) {
      // still allow after start
    }
    if (e.code === 'Space') {
      e.preventDefault();
      if (title.classList.contains('hidden')) $('[data-scoop]').click();
    }
    if (e.key === 's' || e.key === 'S') doSell();
    if (e.key === 'm' || e.key === 'M') {
      game.state.muted = !game.state.muted;
      setMuted(game.state.muted);
      renderSide();
    }
  });

  game.on((evt) => {
    if (evt.type === 'tick') {
      for (const ev of evt.payload) {
        if (ev.type === 'autosell' && ev.ok) {
          spawnFx(`<div class="popnum r-uncommon">auto ${formatCoins(ev.coins)}</div>`, 80, 80);
        }
      }
    }
  });

  renderSide();
  let last = performance.now();
  let shopTimer = 0;
  function frame(now) {
    const dt = Math.min(200, now - last);
    last = now;
    if (game.state.started) {
      const events = game.tick(dt, Math.random, Date.now());
      if (events.some((e) => e.type === 'autosell')) coinSound();
    }
    renderHud(dt);
    shopTimer += dt;
    if (shopTimer > 300) {
      shopTimer = 0;
      renderSide();
    }
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  return {
    resetTitle() {
      title.classList.remove('hidden');
    },
  };
}
