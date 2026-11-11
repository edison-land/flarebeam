<template>
  <div ref="rootRef">
    <h2 class="text-center text-2xl sm:text-3xl font-bold tracking-tight text-white mb-10">未配对的请求，<br class="sm:hidden" />到不了 localhost。</h2>

    <div class="rounded-xl border border-white/[0.08] bg-[#0c0c0f] overflow-hidden">
      <!-- Columns -->
      <div :class="GRID" class="h-10 border-b border-white/[0.06]" aria-hidden="true">
        <div class="relative col-start-2">
          <span
            class="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 text-[11px] font-medium text-[#f38020]"
            :style="{ left: `${GATE}%` }"
          >FlareBeam</span>
          <span class="absolute bottom-0 h-2 w-px bg-[#f38020]/60" :style="{ left: `${GATE}%` }"></span>
        </div>
        <div class="relative">
          <span class="absolute right-0 top-1/2 -translate-y-1/2 whitespace-nowrap font-mono text-[11px] text-emerald-400">
            localhost:{{ port }}
          </span>
        </div>
      </div>

      <!-- Requests -->
      <TransitionGroup
        tag="ol"
        class="relative"
        enter-from-class="opacity-0 -translate-y-1"
        enter-active-class="transition duration-300 ease-out"
        leave-active-class="transition duration-200 absolute inset-x-0"
        leave-to-class="opacity-0"
        move-class="transition-transform duration-300 ease-out"
      >
        <li v-for="row in rows" :key="row.id" :class="GRID" class="h-10 border-b border-white/[0.04] last:border-b-0">
          <div class="flex items-center gap-2.5 min-w-0">
            <component
              :is="row.legit ? Smartphone : Bot"
              class="w-3.5 h-3.5 shrink-0 stroke-[1.75]"
              :class="row.legit ? 'text-emerald-400' : 'text-red-400'"
              role="img"
              :aria-label="row.legit ? '已配对手机' : '扫描器'"
            />
            <span class="font-mono text-xs text-zinc-300 truncate">{{ row.path }}</span>
          </div>

          <div class="relative">
            <div class="absolute inset-x-0 top-1/2 h-px bg-white/[0.07]"></div>
            <div class="absolute inset-y-0 w-px bg-[#f38020]/60" :style="{ left: `${GATE}%` }"></div>
            <div
              class="absolute left-0 top-1/2 -translate-y-1/2 h-[2px] rounded-full transition-[width] ease-[cubic-bezier(0.45,0,0.2,1)]"
              :class="row.legit ? 'bg-gradient-to-r from-transparent to-emerald-400' : 'bg-gradient-to-r from-transparent to-red-500'"
              :style="{ width: row.phase === 'queued' ? '0%' : `${end(row)}%`, transitionDuration: `${travelMs(row)}ms` }"
            ></div>
            <span
              v-if="row.legit ? row.passed : row.phase === 'done'"
              class="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 rounded-full flex items-center justify-center bg-[#0c0c0f] border"
              :class="row.legit ? 'border-emerald-500/60 text-emerald-400' : 'border-red-500 text-red-400'"
              :style="{ left: `${GATE}%` }"
            >
              <Check v-if="row.legit" class="w-2.5 h-2.5 stroke-[3]" />
              <X v-else class="w-2.5 h-2.5 stroke-[3]" />
            </span>
            <span
              v-if="!row.legit && row.phase === 'done'"
              class="absolute top-1/2 -translate-y-1/2 ml-3.5 font-mono text-xs text-red-400"
              :style="{ left: `${GATE}%` }"
            >401</span>
          </div>

          <div class="flex items-center font-mono text-xs text-emerald-400">
            <span v-if="row.legit && row.phase === 'done'">200</span>
          </div>
        </li>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { Smartphone, Bot, Check, X } from 'lucide-vue-next';

withDefaults(defineProps<{ port?: number }>(), { port: 3000 });

const GRID = 'grid grid-cols-[7rem_minmax(0,1fr)_2rem] sm:grid-cols-[12rem_minmax(0,1fr)_2.5rem] gap-x-3 sm:gap-x-5 px-4 sm:px-5';
const GATE = 42; // FlareBeam's position along the track, in %
const MS_PER_PERCENT = 11; // constant speed, so blocked requests stop sooner
const SPAWN_MS = 1200;
const MAX_ROWS = 7;

// `/cart` is in both lists: the path doesn't matter, the session does.
const PHONE_PATHS = ['/', '/api/me', '/checkout', '/cart', '/src/main.ts'];
const SCANNER_PATHS = ['/.env', '/.git/config', '/wp-admin', '/admin', '/cart', '/actuator'];

type Phase = 'queued' | 'travel' | 'done';
interface Row {
  id: number;
  legit: boolean;
  path: string;
  phase: Phase;
  passed: boolean;
}

const rootRef = ref<HTMLElement | null>(null);
const rows = ref<Row[]>([]);

const end = (r: Row) => (r.legit ? 100 : GATE);
const travelMs = (r: Row) => end(r) * MS_PER_PERCENT;
const pick = (list: string[]) => list[Math.floor(Math.random() * list.length)];

let nextId = 0;
let recent: boolean[] = [];

function makeRow(legit: boolean, phase: Phase): Row {
  return { id: nextId++, legit, path: pick(legit ? PHONE_PATHS : SCANNER_PATHS), phase, passed: phase === 'done' };
}

/* Streaming */
const timers = new Set<ReturnType<typeof setTimeout>>();
function later(fn: () => void, ms: number) {
  const t = setTimeout(() => {
    timers.delete(t);
    fn();
  }, ms);
  timers.add(t);
}

function spawn() {
  let legit = Math.random() < 0.5;
  if (recent.length === 2 && recent.every((k) => k === legit)) legit = !legit; // no runs of 3
  recent = [...recent, legit].slice(-2);

  rows.value.unshift(makeRow(legit, 'queued'));
  if (rows.value.length > MAX_ROWS) rows.value.pop();
  const row = rows.value[0]; // reactive proxy

  // paint 0% first, then transition; rAF can lag timers in background tabs, so never go backwards
  requestAnimationFrame(() => requestAnimationFrame(() => row.phase === 'queued' && (row.phase = 'travel')));
  later(() => (row.passed = true), GATE * MS_PER_PERCENT);
  later(() => (row.phase = 'done'), travelMs(row));
}

let loop: ReturnType<typeof setTimeout> | undefined;
let inView = false;
let reducedMotion = false;

function start() {
  if (loop || reducedMotion || !inView || document.hidden) return;
  const tick = () => {
    spawn();
    loop = setTimeout(tick, SPAWN_MS);
  };
  tick();
}
function stop() {
  clearTimeout(loop);
  loop = undefined;
}
const onVisibility = () => (document.hidden ? stop() : start());

let io: IntersectionObserver | null = null;

onMounted(() => {
  reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;
  rows.value = [false, true, false, true, true, false].map((legit) => makeRow(legit, 'done'));

  io = new IntersectionObserver(([e]) => {
    inView = e.isIntersecting;
    inView ? start() : stop();
  });
  if (rootRef.value) io.observe(rootRef.value);
  document.addEventListener('visibilitychange', onVisibility);
});

onUnmounted(() => {
  stop();
  timers.forEach(clearTimeout);
  io?.disconnect();
  document.removeEventListener('visibilitychange', onVisibility);
});
</script>
