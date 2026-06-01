<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import startSfx from "./assets/start.mp3";
import pauseSfx from "./assets/pause.mp3";
import endSfx from "./assets/end.mp3";

const phases = ["red", "yellow", "green"];
const active = ref(0);
const visible = ref(true);
const bgOpacity = ref(1.0);
const muted = ref(false);
let blinkTimer = null;
let unlistenLight = null;
let unlistenTrans = null;
let unlistenMute = null;

const sfx = {
  green: new Audio(startSfx),
  yellow: new Audio(pauseSfx),
  red: new Audio(endSfx),
};

const playSfx = (color) => {
  if (muted.value) return;
  const audio = sfx[color];
  if (!audio) return;
  audio.pause();
  audio.currentTime = 0;
  audio.play().catch(() => {});
};

onMounted(async () => {
  try {
    const snapshot = await invoke("get_app_state");
    if (snapshot && typeof snapshot.transparency === "number") {
      bgOpacity.value = snapshot.transparency;
    }
    if (snapshot && typeof snapshot.muted === "boolean") {
      muted.value = snapshot.muted;
    }
  } catch (e) {
    // ignore — fall back to defaults
  }

  unlistenLight = await listen("traffic-light", (event) => {
    const { color, interval } = event.payload ?? {};
    const idx = phases.indexOf(color);
    if (idx < 0) return;
    const changed = active.value !== idx;
    active.value = idx;
    visible.value = true;
    if (blinkTimer) {
      clearInterval(blinkTimer);
      blinkTimer = null;
    }
    if (changed) playSfx(color);
    const ms = Number(interval);
    if (Number.isFinite(ms) && ms > 0) {
      blinkTimer = setInterval(() => {
        visible.value = !visible.value;
      }, ms);
    }
  });

  unlistenTrans = await listen("traffic-transparency", (event) => {
    const { transparency } = event.payload ?? {};
    if (typeof transparency === "number" && transparency >= 0 && transparency <= 1) {
      bgOpacity.value = transparency;
    }
  });

  unlistenMute = await listen("traffic-mute", (event) => {
    const { muted: next } = event.payload ?? {};
    if (typeof next === "boolean") {
      muted.value = next;
    }
  });
});

onUnmounted(() => {
  if (blinkTimer) clearInterval(blinkTimer);
  if (unlistenLight) unlistenLight();
  if (unlistenTrans) unlistenTrans();
  if (unlistenMute) unlistenMute();
});
</script>

<template>
  <main class="container" data-tauri-drag-region>
    <div
      class="traffic-light"
      :style="{ '--traffic-bg-opacity': bgOpacity }"
      data-tauri-drag-region
  >
      <div
        v-for="(color, i) in phases"
        :key="color"
        class="lamp"
        :class="[color, { on: active === i, dim: active === i && !visible }]"
        data-tauri-drag-region
      ></div>
    </div>
  </main>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: transparent;
}

.container {
  box-sizing: border-box;
  height: 100vh;
  width: 100vw;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  overflow: hidden;
  cursor: grab;
}

.container:active {
  cursor: grabbing;
}

.traffic-light {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  width: 56px;
  height: 150px;
  padding: 8px 0;
  --traffic-bg-opacity: 1;
  background: linear-gradient(
    145deg,
    rgba(42, 42, 42, var(--traffic-bg-opacity)),
    rgba(17, 17, 17, var(--traffic-bg-opacity))
  );
  border-radius: 14px;
  box-shadow: inset 0 2px 3px rgba(255, 255, 255, calc(0.08 * var(--traffic-bg-opacity))),
    inset 0 -2px 3px rgba(0, 0, 0, calc(0.6 * var(--traffic-bg-opacity))),
    0 4px 10px rgba(0, 0, 0, calc(0.45 * var(--traffic-bg-opacity)));
}

.lamp {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: radial-gradient(
    circle at 30% 30%,
    rgba(255, 255, 255, 0.18),
    rgba(0, 0, 0, 0.55) 70%
  );
  box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.1),
    inset 0 -2px 4px rgba(0, 0, 0, 0.6);
  transition: background 0.35s ease, box-shadow 0.35s ease, filter 0.35s ease;
  filter: brightness(0.55) saturate(0.7);
}

.lamp.red.on {
  background: radial-gradient(circle at 30% 30%, #ffb3b3, #ff2a2a 55%, #6b0000 100%);
  box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.45),
    inset 0 -3px 6px rgba(0, 0, 0, 0.5), 0 0 14px 4px rgba(255, 60, 60, 0.65),
    0 0 30px 10px rgba(255, 60, 60, 0.35);
  filter: brightness(1.15) saturate(1.2);
}

.lamp.yellow.on {
  background: radial-gradient(circle at 30% 30%, #fff4b3, #ffcc33 55%, #6b4a00 100%);
  box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.5),
    inset 0 -3px 6px rgba(0, 0, 0, 0.5), 0 0 14px 4px rgba(255, 204, 51, 0.7),
    0 0 30px 10px rgba(255, 204, 51, 0.35);
  filter: brightness(1.15) saturate(1.2);
}

.lamp.green.on {
  background: radial-gradient(circle at 30% 30%, #c8ffc8, #2ecc4f 55%, #003d10 100%);
  box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.45),
    inset 0 -3px 6px rgba(0, 0, 0, 0.5), 0 0 14px 4px rgba(46, 204, 79, 0.65),
    0 0 30px 10px rgba(46, 204, 79, 0.35);
  filter: brightness(1.15) saturate(1.2);
}

.lamp.dim {
  opacity: 0.08;
  transition: opacity 0s;
}
</style>
