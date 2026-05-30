<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";

const phases = ["red", "yellow", "green"];
const active = ref(0);
const visible = ref(true);
let blinkTimer = null;
let unlisten = null;

onMounted(async () => {
  unlisten = await listen("traffic-light", (event) => {
    const { color, interval } = event.payload ?? {};
    const idx = phases.indexOf(color);
    if (idx < 0) return;
    active.value = idx;
    visible.value = true;
    if (blinkTimer) {
      clearInterval(blinkTimer);
      blinkTimer = null;
    }
    const ms = Number(interval);
    if (Number.isFinite(ms) && ms > 0) {
      blinkTimer = setInterval(() => {
        visible.value = !visible.value;
      }, ms);
    }
  });
});

onUnmounted(() => {
  if (blinkTimer) clearInterval(blinkTimer);
  if (unlisten) unlisten();
});
</script>

<template>
  <main class="container" data-tauri-drag-region>
    <div class="traffic-light" data-tauri-drag-region>
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
  background: linear-gradient(145deg, #2a2a2a, #111);
  border-radius: 14px;
  box-shadow: inset 0 2px 3px rgba(255, 255, 255, 0.08),
    inset 0 -2px 3px rgba(0, 0, 0, 0.6), 0 4px 10px rgba(0, 0, 0, 0.45);
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
