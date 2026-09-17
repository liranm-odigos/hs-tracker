/** Funny SVG actors. Not AAA. Definitely a raccoon with a tiny tie. */

export const RACCOON_SVG = `
<svg class="raccoon-svg" viewBox="0 0 170 180" aria-hidden="true">
  <ellipse class="r-shadow" cx="84" cy="168" rx="48" ry="10" fill="#000" opacity=".28"/>
  <g class="r-tail-w">
    <g class="r-tail" transform="translate(18,108)">
      <ellipse cx="0" cy="10" rx="22" ry="12" fill="#6b5848" transform="rotate(-30)"/>
      <ellipse cx="-6" cy="6" rx="10" ry="7" fill="#2a2430" transform="rotate(-30)"/>
    </g>
  </g>
  <g class="r-body">
    <g class="r-arm-l">
      <g transform="translate(34,108)">
        <ellipse cx="0" cy="8" rx="13" ry="9" fill="#6b5848"/>
        <ellipse cx="-4" cy="10" rx="8" ry="6" fill="#cbb7a0"/>
      </g>
    </g>
    <ellipse cx="84" cy="122" rx="42" ry="36" fill="#6b5848"/>
    <ellipse cx="84" cy="132" rx="28" ry="16" fill="#cbb7a0"/>
    <g class="r-tie">
      <g transform="translate(84,124)">
        <rect x="-6" y="0" width="12" height="28" rx="3" fill="#2a1c10"/>
        <rect x="-8" y="0" width="16" height="8" rx="2" fill="#ffd166"/>
      </g>
    </g>
    <g class="r-arm-r">
      <g transform="translate(134,108)">
        <ellipse cx="0" cy="8" rx="13" ry="9" fill="#6b5848"/>
        <ellipse cx="5" cy="10" rx="8" ry="6" fill="#cbb7a0"/>
      </g>
    </g>
    <g class="r-head-w">
      <g class="r-head" transform="translate(84,70)">
      <ellipse class="r-ear" cx="-28" cy="-28" rx="14" ry="16" fill="#6b5848"/>
      <ellipse class="r-ear" cx="28" cy="-28" rx="14" ry="16" fill="#6b5848"/>
      <ellipse cx="-28" cy="-28" rx="8" ry="10" fill="#2a2430"/>
      <ellipse cx="28" cy="-28" rx="8" ry="10" fill="#2a2430"/>
      <circle cx="0" cy="-6" r="32" fill="#6b5848"/>
      <ellipse cx="-16" cy="-6" rx="16" ry="12" fill="#2a2430"/>
      <ellipse cx="16" cy="-6" rx="16" ry="12" fill="#2a2430"/>
      <g class="r-eye" transform="translate(-16,-6)">
        <circle r="5" fill="#f7f1e8"/>
        <circle class="r-pupil" cx="1" cy="1" r="2.2" fill="#1a1224"/>
        <rect class="r-lid" x="-6" y="-8" width="12" height="8" rx="3" fill="#6b5848"/>
      </g>
      <g class="r-eye" transform="translate(16,-6)">
        <circle r="5" fill="#f7f1e8"/>
        <circle class="r-pupil" cx="1" cy="1" r="2.2" fill="#1a1224"/>
        <rect class="r-lid" x="-6" y="-8" width="12" height="8" rx="3" fill="#6b5848"/>
      </g>
      <ellipse cx="0" cy="8" rx="6" ry="4" fill="#1a1224"/>
      <path d="M-6 14 Q0 20 6 14" fill="none" stroke="#1a1224" stroke-width="2"/>
      </g>
    </g>
    <ellipse cx="70" cy="156" rx="11" ry="7" fill="#2a2430"/>
    <ellipse cx="100" cy="156" rx="11" ry="7" fill="#2a2430"/>
  </g>
</svg>`;

export const GARY_SVG = `
<svg class="gary-svg" viewBox="0 0 90 110" aria-hidden="true">
  <ellipse cx="45" cy="102" rx="22" ry="6" fill="#000" opacity=".25"/>
  <ellipse cx="45" cy="78" rx="22" ry="20" fill="#3a2a20"/>
  <rect x="28" y="62" width="34" height="28" rx="4" fill="#1a1220"/>
  <circle cx="45" cy="44" r="20" fill="#6b5848"/>
  <ellipse cx="36" cy="44" rx="8" ry="6" fill="#2a2430"/>
  <ellipse cx="54" cy="44" rx="8" ry="6" fill="#2a2430"/>
  <circle cx="37" cy="44" r="2.4" fill="#f7f1e8"/>
  <circle cx="53" cy="44" r="2.4" fill="#f7f1e8"/>
  <ellipse cx="45" cy="52" rx="4" ry="3" fill="#1a1224"/>
  <rect x="18" y="58" width="16" height="6" rx="2" fill="#c9a227"/>
  <path d="M20 58 Q12 48 22 44" fill="none" stroke="#c9a227" stroke-width="3"/>
</svg>`;

export const CAT_SVG = `
<svg class="cat-svg" viewBox="0 0 90 70" aria-hidden="true">
  <ellipse cx="48" cy="48" rx="28" ry="16" fill="#e8b86a"/>
  <circle cx="28" cy="30" r="16" fill="#e8b86a"/>
  <polygon points="16,22 20,8 28,20" fill="#e8b86a"/>
  <polygon points="40,22 36,8 28,20" fill="#e8b86a"/>
  <circle cx="23" cy="30" r="2" fill="#1a1224"/>
  <circle cx="33" cy="30" r="2" fill="#1a1224"/>
  <ellipse cx="28" cy="36" rx="3" ry="2" fill="#ff6b9d"/>
  <path class="cat-tail" d="M74 48 Q90 20 78 12" fill="none" stroke="#e8b86a" stroke-width="6" stroke-linecap="round"/>
</svg>`;

export const POSSUM_SVG = `
<svg class="possum-svg" viewBox="0 0 90 70" aria-hidden="true">
  <ellipse cx="50" cy="46" rx="26" ry="16" fill="#cfc6b8"/>
  <circle cx="28" cy="32" r="16" fill="#cfc6b8"/>
  <ellipse cx="18" cy="38" rx="8" ry="5" fill="#f4b6c2"/>
  <circle cx="24" cy="30" r="2" fill="#1a1224"/>
  <circle cx="32" cy="30" r="2" fill="#1a1224"/>
  <path d="M70 46 Q86 28 80 16" fill="none" stroke="#cfc6b8" stroke-width="5" stroke-linecap="round"/>
  <text x="20" y="18" font-size="10" fill="#1a1224">z</text>
</svg>`;
