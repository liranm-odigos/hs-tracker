import { TREE_NODES, TREE_SIZE, affordableTreeCount, nextUnlockHint, treeView } from './tree.js';
import { formatCoins } from './format.js';

function el(html) {
  const t = document.createElement('template');
  t.innerHTML = html.trim();
  return t.content.firstElementChild;
}

export function createScheme(root, game, { onBuy, onClose }) {
  const wrap = el(`
    <div class="scheme hidden" data-scheme>
      <div class="scheme-bar">
        <div>
          <h2>THE SCHEME</h2>
          <p data-scheme-sub>Spend coins. Unlock worse ideas.</p>
        </div>
        <button class="btn" data-scheme-close>Close · T</button>
      </div>
      <div class="scheme-port" data-port>
        <div class="scheme-world" data-world style="width:${TREE_SIZE.w}px;height:${TREE_SIZE.h}px">
          <svg class="scheme-lines" viewBox="0 0 ${TREE_SIZE.w} ${TREE_SIZE.h}" data-lines></svg>
          <div data-nodes></div>
        </div>
      </div>
      <div class="scheme-tip" data-tip>Click a glowing node. Parents unlock children. Spending unlocks whole branches.</div>
    </div>`);
  root.append(wrap);

  const port = wrap.querySelector('[data-port]');
  const world = wrap.querySelector('[data-world]');
  const lines = wrap.querySelector('[data-lines]');
  const nodesEl = wrap.querySelector('[data-nodes]');
  const tip = wrap.querySelector('[data-tip]');
  const sub = wrap.querySelector('[data-scheme-sub]');

  let drag = null;
  port.addEventListener('mousedown', (e) => {
    if (e.target.closest('.s-node')) return;
    drag = { x: e.clientX, y: e.clientY, sl: port.scrollLeft, st: port.scrollTop };
  });
  window.addEventListener('mousemove', (e) => {
    if (!drag) return;
    port.scrollLeft = drag.sl - (e.clientX - drag.x);
    port.scrollTop = drag.st - (e.clientY - drag.y);
  });
  window.addEventListener('mouseup', () => {
    drag = null;
  });

  wrap.querySelector('[data-scheme-close]').addEventListener('click', () => close());

  nodesEl.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-node]');
    if (!btn) return;
    const id = btn.getAttribute('data-node');
    const r = game.buy('tree', id);
    onBuy?.(r);
    render();
    if (r.ok && r.opened?.length) {
      const names = r.opened.map((n) => n.name).join(', ');
      tip.textContent = `Unlocked: ${names}. Keep spending.`;
      tip.classList.add('hot');
    }
  });

  nodesEl.addEventListener('mouseover', (e) => {
    const btn = e.target.closest('[data-node]');
    if (!btn) return;
    const id = btn.getAttribute('data-node');
    const node = TREE_NODES.find((n) => n.id === id);
    const row = treeView(game.state).find((x) => x.node.id === id);
    if (!node || !row) return;
    const lock = row.unlocked ? node.desc : nextUnlockHint(game.state, node);
    tip.classList.remove('hot');
    tip.innerHTML = `<b>${node.icon} ${node.name}</b> · Lv ${row.level}${node.max ? '/' + node.max : ''} · ${row.maxed ? 'MAX' : formatCoins(row.cost)}<br>${lock}`;
  });

  function render() {
    const s = game.state;
    const rows = treeView(s);
    const byId = Object.fromEntries(rows.map((r) => [r.node.id, r]));
    sub.textContent = `Spent ${formatCoins(s.spent || 0)} on schemes · ${affordableTreeCount(s)} ready to buy`;

    lines.innerHTML = TREE_NODES.flatMap((n) =>
      (n.parents || []).map((pid) => {
        const p = TREE_NODES.find((x) => x.id === pid);
        if (!p) return '';
        const child = byId[n.id];
        const parent = byId[pid];
        const on = parent && parent.level > 0;
        return `<line x1="${p.x}" y1="${p.y + 36}" x2="${n.x}" y2="${n.y - 36}" class="${on ? 'lit' : ''} ${child?.unlocked ? 'open' : ''}" />`;
      }),
    ).join('');

    nodesEl.innerHTML = rows
      .map((row) => {
        const { node, level, unlocked, visible, maxed, affordable, cost } = row;
        if (!visible && !node.start) return '';
        const cls = [
          's-node',
          node.start || level > 0 ? 'owned' : '',
          unlocked && !maxed && !node.start ? 'unlocked' : '',
          affordable ? 'buyable' : '',
          maxed ? 'maxed' : '',
          !unlocked && !node.start ? 'locked' : '',
        ].join(' ');
        const badge = node.start ? 'YOU' : maxed ? 'MAX' : `Lv ${level}`;
        return `<button class="${cls}" data-node="${node.id}" style="left:${node.x}px;top:${node.y}px" ${unlocked && !maxed && !node.start ? '' : 'disabled'}>
          <span class="s-ico">${unlocked || level > 0 || node.start ? node.icon : '❓'}</span>
          <span class="s-name">${unlocked || level > 0 ? node.name : '???'}</span>
          <span class="s-meta">${badge}${!maxed && !node.start ? ' · ' + formatCoins(cost) : ''}</span>
        </button>`;
      })
      .join('');
  }

  function open() {
    wrap.classList.remove('hidden');
    render();
    const rootNode = TREE_NODES.find((n) => n.id === 'root');
    port.scrollLeft = Math.max(0, rootNode.x - port.clientWidth / 2);
    port.scrollTop = 0;
  }

  function close() {
    wrap.classList.add('hidden');
    onClose?.();
  }

  function isOpen() {
    return !wrap.classList.contains('hidden');
  }

  return { wrap, open, close, render, isOpen };
}
