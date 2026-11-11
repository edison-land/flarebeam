<template>
  <canvas ref="canvasRef" class="absolute inset-0 pointer-events-none z-0"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = defineProps<{
  active: boolean;
  paired: boolean;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
let animationFrameId: number | null = null;

interface Ripple {
  x: number;
  y: number;
  radius: number;
  maxRadius: number;
  alpha: number;
  speed: number;
  color: string;
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  alpha: number;
  maxLife: number;
  life: number;
  color: string;
}

let ripples: Ripple[] = [];
let particles: Particle[] = [];
let lastRippleTime = 0;

function createRipple(cx: number, cy: number, color: string = 'rgba(243, 128, 32, 0.4)') {
  ripples.push({
    x: cx,
    y: cy,
    radius: 10,
    maxRadius: 180 + Math.random() * 80,
    alpha: 0.6,
    speed: 1.8 + Math.random() * 1.2,
    color,
  });
}

function spawnBurst(cx: number, cy: number, count: number = 36, isEmerald: boolean = false) {
  for (let i = 0; i < count; i++) {
    const angle = (Math.PI * 2 * i) / count + (Math.random() - 0.5) * 0.4;
    const speed = 1.5 + Math.random() * 3.5;
    const colors = isEmerald
      ? ['#10b981', '#34d399', '#6ee7b7', '#a7f3d0']
      : ['#f38020', '#fb923c', '#fcd34d', '#38bdf8'];
    const color = colors[Math.floor(Math.random() * colors.length)];
    particles.push({
      x: cx,
      y: cy,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed,
      radius: 1.2 + Math.random() * 2,
      alpha: 0.8 + Math.random() * 0.2,
      maxLife: 60 + Math.random() * 40,
      life: 0,
      color,
    });
  }
}

function spawnAmbient(width: number, height: number) {
  if (particles.length > 50) return;
  particles.push({
    x: Math.random() * width,
    y: Math.random() * height,
    vx: (Math.random() - 0.5) * 0.4,
    vy: (Math.random() - 0.5) * 0.4 - 0.2,
    radius: 0.8 + Math.random() * 1.4,
    alpha: 0.15 + Math.random() * 0.25,
    maxLife: 120 + Math.random() * 80,
    life: 0,
    color: '#f38020',
  });
}

function render(now: number) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;
  const dpr = window.devicePixelRatio || 1;
  const cx = (w / dpr) / 2;
  const cy = (h / dpr) / 2 - 30; // centered slightly above for QR code

  // Soft trail effect for motion blur
  ctx.clearRect(0, 0, w, h);

  // Periodic ripples
  if (props.active && now - lastRippleTime > 1400) {
    createRipple(
      cx,
      cy,
      props.paired ? 'rgba(16, 185, 129, 0.45)' : 'rgba(243, 128, 32, 0.35)'
    );
    lastRippleTime = now;
  }

  // Draw & update Ripples (水波纹)
  for (let i = ripples.length - 1; i >= 0; i--) {
    const r = ripples[i];
    r.radius += r.speed;
    const progress = r.radius / r.maxRadius;
    const currentAlpha = (1 - progress) * r.alpha;

    if (progress >= 1) {
      ripples.splice(i, 1);
      continue;
    }

    ctx.save();
    ctx.beginPath();
    ctx.arc(r.x, r.y, r.radius, 0, Math.PI * 2);
    ctx.strokeStyle = r.color.replace(/[\d.]+\)$/, `${currentAlpha})`);
    ctx.lineWidth = 1.5 * (1 - progress * 0.6);
    ctx.stroke();

    // Subtle radial gradient fill inside the ripple
    if (r.radius < 60) {
      const grad = ctx.createRadialGradient(r.x, r.y, 0, r.x, r.y, r.radius);
      grad.addColorStop(0, r.color.replace(/[\d.]+\)$/, `${currentAlpha * 0.2})`));
      grad.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.fillStyle = grad;
      ctx.fill();
    }
    ctx.restore();
  }

  // Draw & update Particles (粒子)
  if (props.active && Math.random() < 0.25) {
    spawnAmbient(w / dpr, h / dpr);
  }

  for (let i = particles.length - 1; i >= 0; i--) {
    const p = particles[i];
    p.life++;
    p.x += p.vx;
    p.y += p.vy;
    p.vx *= 0.985;
    p.vy *= 0.985;

    const progress = p.life / p.maxLife;
    const currentAlpha = (1 - progress) * p.alpha;

    if (progress >= 1) {
      particles.splice(i, 1);
      continue;
    }

    ctx.save();
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
    ctx.fillStyle = p.color;
    ctx.globalAlpha = currentAlpha;
    ctx.shadowBlur = 6;
    ctx.shadowColor = p.color;
    ctx.fill();
    ctx.restore();
  }

  if (props.active || ripples.length > 0 || particles.length > 0) {
    animationFrameId = requestAnimationFrame(render);
  }
}

let introTimer: ReturnType<typeof setTimeout> | undefined;
let resizeObs: ResizeObserver | null = null;

function introBurst() {
  const canvas = canvasRef.value;
  if (!canvas || !props.active) return;
  const dpr = window.devicePixelRatio || 1;
  const cx = (canvas.width / dpr) / 2;
  const cy = (canvas.height / dpr) / 2 - 30;
  createRipple(cx, cy, 'rgba(243, 128, 32, 0.6)');
  createRipple(cx, cy, 'rgba(251, 191, 36, 0.4)');
  spawnBurst(cx, cy, 48, false);
}

function resizeCanvas() {
  const canvas = canvasRef.value;
  if (!canvas || !canvas.parentElement) return;
  const rect = canvas.parentElement.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  const ctx = canvas.getContext('2d');
  if (ctx) {
    ctx.scale(dpr, dpr);
  }
}

watch(
  () => props.active,
  (active) => {
    if (active) {
      resizeCanvas();
      // On desktop the panel slides open from 0 width (500ms); burst once it has its final size.
      const opening = (canvasRef.value?.parentElement?.clientWidth ?? 0) < 50;
      introTimer = setTimeout(introBurst, opening ? 520 : 0);
      if (!animationFrameId) {
        animationFrameId = requestAnimationFrame(render);
      }
    } else {
      clearTimeout(introTimer);
      ripples = [];
      particles = [];
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
      const canvas = canvasRef.value;
      if (canvas) {
        const ctx = canvas.getContext('2d');
        ctx?.clearRect(0, 0, canvas.width, canvas.height);
      }
    }
  },
  { flush: 'post' }
);

watch(
  () => props.paired,
  (paired) => {
    if (paired) {
      const canvas = canvasRef.value;
      if (canvas) {
        const dpr = window.devicePixelRatio || 1;
        const cx = (canvas.width / dpr) / 2;
        const cy = (canvas.height / dpr) / 2 - 30;
        createRipple(cx, cy, 'rgba(16, 185, 129, 0.8)');
        spawnBurst(cx, cy, 60, true);
      }
    }
  }
);

onMounted(() => {
  resizeCanvas();
  resizeObs = new ResizeObserver(resizeCanvas);
  if (canvasRef.value?.parentElement) resizeObs.observe(canvasRef.value.parentElement);
  if (props.active) {
    animationFrameId = requestAnimationFrame(render);
  }
});

onUnmounted(() => {
  clearTimeout(introTimer);
  resizeObs?.disconnect();
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }
});
</script>
