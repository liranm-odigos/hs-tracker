import './style.css';
import { Game } from './game.js';
import { mount } from './ui.js';
import { clearSave, loadSave, writeSave } from './save.js';
import { setMuted } from './audio.js';

const app = document.getElementById('app');
let game = new Game(loadSave());
setMuted(game.state.muted);

function boot() {
  app.innerHTML = '';
  mount(app, game);
}

boot();

app.addEventListener('new-game', () => {
  clearSave();
  game = new Game(null);
  game.state.started = true;
  setMuted(false);
  boot();
  app.querySelector('[data-title]')?.classList.add('hidden');
});

setInterval(() => {
  if (game.state.started) writeSave(game.state);
}, 1500);

window.addEventListener('beforeunload', () => {
  if (game.state.started) writeSave(game.state);
});
