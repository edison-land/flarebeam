<template>
  <div class="h-screen w-screen flex flex-col bg-[#09090b] text-zinc-100 select-none font-sans overflow-hidden antialiased">
    <!-- Window Header -->
    <header class="h-11 border-b border-white/[0.06] flex items-center justify-between px-5 bg-[#0c0c0e]/80 backdrop-blur z-20 shrink-0">
      <div class="flex items-center gap-2.5">
        <div class="w-2 h-2 rounded-full transition-colors"
             :class="status.is_active 
               ? (status.local_alive ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.6)]' : 'bg-red-400') 
               : 'bg-zinc-600'"></div>
        <span class="font-semibold text-xs tracking-tight text-zinc-200">FlareBeam</span>
      </div>

      <!-- Header Live Status (Quiet when idle) -->
      <div v-if="status.is_active" class="flex items-center gap-2 text-[11px] font-mono text-zinc-400">
        <span v-if="status.paired" class="text-emerald-400 flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
          手机已连通
        </span>
        <span v-else-if="status.local_alive" class="text-amber-400/90 flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse"></span>
          等待扫码
        </span>
        <span v-else class="text-red-400 flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-red-400"></span>
          服务异常
        </span>
        <span class="text-zinc-600">·</span>
        <span>{{ status.request_count }} 次请求</span>
      </div>
    </header>

    <!-- Main Canvas -->
    <main class="flex-1 flex overflow-hidden">
      <!-- Left Panel: Minimal Controls (Always 360px) -->
      <section 
        class="w-[360px] p-6 flex flex-col justify-between shrink-0 bg-[#0c0c0e] h-full overflow-hidden z-10 transition-colors"
        :class="status.is_active ? 'border-r border-white/[0.06]' : ''">
        <div class="space-y-6 flex-1 overflow-y-auto pr-1">
          <!-- Port Selection -->
          <div class="space-y-2.5">
            <div class="flex items-center justify-between">
              <label class="text-[11px] font-medium text-zinc-400 tracking-wider">本地端口</label>
              <button 
                @click="scanPorts" 
                :disabled="isScanning || status.is_active"
                class="text-zinc-500 hover:text-zinc-300 transition-colors p-1 -mr-1"
                title="重新扫描活跃端口">
                <RefreshCw class="w-3.5 h-3.5" :class="{'animate-spin': isScanning}" />
              </button>
            </div>

            <!-- Unified Localhost Input Box -->
            <div class="flex items-center bg-[#151518] border border-white/[0.08] focus-within:border-white/20 rounded-lg px-3.5 py-2.5 transition">
              <span class="text-xs font-mono text-zinc-500 mr-2 select-none">localhost:</span>
              <input 
                v-model.number="selectedPort" 
                type="number" 
                placeholder="3000" 
                :disabled="status.is_active"
                class="w-full bg-transparent text-sm font-mono text-white outline-none placeholder-zinc-600 disabled:opacity-40" />
            </div>

            <!-- Detected Active Ports Chips -->
            <div v-if="activePorts.length > 0" class="flex flex-wrap gap-2 pt-0.5">
              <button 
                v-for="p in activePorts" 
                :key="p.port"
                @click="selectedPort = p.port; errorMsg = ''"
                :disabled="status.is_active"
                class="text-xs font-mono px-3 py-1.5 rounded-md border transition-all flex items-center gap-1.5"
                :class="selectedPort === p.port 
                  ? 'bg-white/10 border-white/20 text-white font-medium shadow-sm' 
                  : 'bg-[#151518]/70 border-white/[0.06] text-zinc-400 hover:text-zinc-200 hover:border-white/10'">
                <span>:{{ p.port }}</span>
                <span class="text-[10px] text-zinc-500 font-sans">{{ p.name }}</span>
              </button>
            </div>
          </div>

          <!-- Security Mode (Linear-style Segmented Control) -->
          <div class="space-y-2.5">
            <label class="text-[11px] font-medium text-zinc-400 tracking-wider">访问权限</label>
            <div class="grid grid-cols-2 gap-1 p-1 bg-[#151518] border border-white/[0.08] rounded-lg">
              <button 
                type="button"
                @click="isPublic = false; errorMsg = ''"
                :disabled="status.is_active"
                class="py-2 px-3 rounded-md text-xs font-medium transition-all flex items-center justify-center gap-1.5"
                :class="!isPublic 
                  ? 'bg-[#222226] text-white shadow-sm' 
                  : 'text-zinc-400 hover:text-zinc-200'">
                <Lock class="w-3.5 h-3.5 text-amber-500" />
                <span>PIN 保护</span>
              </button>
              <button 
                type="button"
                @click="isPublic = true; errorMsg = ''"
                :disabled="status.is_active"
                class="py-2 px-3 rounded-md text-xs font-medium transition-all flex items-center justify-center gap-1.5"
                :class="isPublic 
                  ? 'bg-[#222226] text-white shadow-sm' 
                  : 'text-zinc-400 hover:text-zinc-200'">
                <Globe class="w-3.5 h-3.5 text-zinc-300" />
                <span>公开访问</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Action Area (Always Fixed at Bottom) -->
        <div class="pt-4 border-t border-white/[0.06] shrink-0">
          <div v-if="errorMsg" class="mb-3 p-3 rounded-lg bg-red-950/40 border border-red-500/20 text-xs text-red-300 flex items-start justify-between gap-2">
            <span class="leading-relaxed">{{ errorMsg }}</span>
            <button @click="errorMsg = ''" class="text-red-400/80 hover:text-red-200 p-0.5">✕</button>
          </div>

          <button 
            v-if="!status.is_active"
            @click="startPreview"
            :disabled="isLaunching || !selectedPort"
            class="w-full h-11 rounded-lg bg-[#f38020] hover:bg-[#e06f15] text-white font-medium text-xs tracking-wide transition-all duration-150 flex items-center justify-center gap-2 shadow-sm active:scale-[0.99] disabled:opacity-40 disabled:cursor-not-allowed">
            <RefreshCw v-if="isLaunching" class="w-4 h-4 animate-spin text-white/90" />
            <span>{{ isLaunching ? '正在连接边缘网络...' : '开启预览' }}</span>
          </button>

          <button 
            v-else
            @click="stopPreview"
            :disabled="isLaunching"
            class="w-full h-11 rounded-lg bg-[#18181b] hover:bg-[#202024] border border-white/10 text-zinc-300 hover:text-red-400 font-medium text-xs tracking-wide transition-all flex items-center justify-center gap-2">
            <Power class="w-4 h-4" />
            <span>停止预览</span>
          </button>
        </div>
      </section>

      <!-- Right Panel: Dynamically Expanded Canvas with Quantum Particle & Ripple Shockwaves -->
      <section 
        class="flex-1 flex flex-col items-center justify-center bg-[#09090b] relative overflow-hidden transition-all duration-500 ease-[cubic-bezier(0.16,1,0.3,1)]"
        :class="status.is_active ? 'opacity-100 p-8' : 'w-0 max-w-0 p-0 opacity-0 pointer-events-none'">
        
        <!-- Particle & Ripple Canvas Background -->
        <BeamCanvas :active="status.is_active" :paired="status.paired" />

        <!-- Floating QR Card & Live Details with Spring Pop -->
        <div v-if="status.is_active" class="w-full max-w-sm flex flex-col items-center space-y-6 z-10 animate-spring-pop">
          <!-- Server Warning if target port dies -->
          <div v-if="!status.local_alive" class="w-full p-3 rounded-lg bg-red-950/40 border border-red-500/20 text-red-300 text-xs flex items-center gap-2 animate-pulse">
            <AlertCircle class="w-4 h-4 shrink-0 text-red-400" />
            <span>本地服务已停止响应 (127.0.0.1:{{ status.target_port }})</span>
          </div>

          <!-- Spring-loaded Floating White QR Card -->
          <div class="p-4 bg-white rounded-2xl shadow-2xl relative shadow-orange-950/30 ring-1 ring-white/20">
            <img v-if="qrDataUrl" :src="qrDataUrl" alt="QR Code" class="w-52 h-52 block" />
            <div v-else class="w-52 h-52 flex items-center justify-center text-zinc-400 text-xs">
              生成二维码...
            </div>

            <!-- Paired Connected Badge -->
            <div v-if="status.paired" class="absolute -bottom-3 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full bg-emerald-500 text-zinc-950 text-[11px] font-semibold tracking-wide shadow-lg whitespace-nowrap flex items-center gap-1.5 animate-bounce">
              <span class="w-1.5 h-1.5 rounded-full bg-zinc-950"></span>
              手机已连通
            </div>
          </div>

          <!-- PIN Code Display with Staggered Flip -->
          <div v-if="status.pin" class="flex flex-col items-center space-y-2 pt-1">
            <span class="text-[11px] font-mono uppercase tracking-widest text-zinc-500 font-medium">
              验证 PIN 码
            </span>
            <div class="flex gap-2">
              <div 
                v-for="(digit, idx) in status.pin.split('')" 
                :key="idx"
                :style="{ animationDelay: `${idx * 60 + 200}ms` }"
                class="w-11 h-13 rounded-xl bg-[#141416] border border-white/[0.08] text-[#f38020] font-mono text-2xl font-bold flex items-center justify-center shadow-inner animate-digit-flip">
                {{ digit }}
              </div>
            </div>
          </div>

          <!-- Public URL Bar -->
          <div class="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-[#141416] border border-white/[0.08]">
            <span class="flex-1 text-xs font-mono text-zinc-400 truncate select-all px-1">
              {{ status.url }}
            </span>
            <button 
              @click="copyUrl" 
              class="p-1.5 rounded-md hover:bg-white/10 text-zinc-400 hover:text-white transition-colors"
              :title="copied ? '已复制' : '复制链接'">
              <Check v-if="copied" class="w-3.5 h-3.5 text-emerald-400" />
              <Copy v-else class="w-3.5 h-3.5" />
            </button>
            <a 
              :href="status.url || undefined" 
              target="_blank" 
              class="p-1.5 rounded-md hover:bg-white/10 text-zinc-400 hover:text-white transition-colors"
              title="在浏览器中打开">
              <ExternalLink class="w-3.5 h-3.5" />
            </a>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import QRCode from 'qrcode';
import {
  RefreshCw,
  Lock,
  Globe,
  Power,
  AlertCircle,
  Check,
  Copy,
  ExternalLink,
} from 'lucide-vue-next';
import BeamCanvas from './components/BeamCanvas.vue';

interface PreviewStatus {
  is_active: boolean;
  url: string | null;
  pin: string | null;
  target_port: number | null;
  local_alive: boolean;
  paired: boolean;
  request_count: number;
  is_public: boolean;
}

interface DiscoveredPort {
  port: number;
  name: string;
}

const selectedPort = ref<number>(3000);
const activePorts = ref<DiscoveredPort[]>([]);
const isPublic = ref<boolean>(false);
const isScanning = ref<boolean>(false);
const isLaunching = ref<boolean>(false);
const errorMsg = ref<string>('');
const copied = ref<boolean>(false);
const qrDataUrl = ref<string>('');

const status = ref<PreviewStatus>({
  is_active: false,
  url: null,
  pin: null,
  target_port: null,
  local_alive: false,
  paired: false,
  request_count: 0,
  is_public: false,
});

watch([selectedPort, isPublic], () => {
  errorMsg.value = '';
});

let pollTimer: number | null = null;

async function scanPorts() {
  isScanning.value = true;
  try {
    const ports = await invoke<DiscoveredPort[]>('scan_ports');
    activePorts.value = ports;
    if (ports.length > 0 && !status.value.is_active && !ports.some(p => p.port === selectedPort.value)) {
      selectedPort.value = ports[0].port;
    }
  } catch (err) {
    console.error('Failed to scan ports:', err);
  } finally {
    isScanning.value = false;
  }
}

async function startPreview() {
  if (!selectedPort.value) {
    errorMsg.value = '请输入有效的端口号';
    return;
  }
  isLaunching.value = true;
  errorMsg.value = '';

  try {
    // 1. Establish Cloudflare tunnel & proxy silently while window remains compact
    const info = await invoke<{
      url: string;
      pin: string | null;
      target_port: number;
      proxy_port: number;
      is_public: boolean;
    }>('start_preview', {
      targetPort: selectedPort.value,
      isPublic: isPublic.value,
    });

    // 2. Pre-generate the QR Code image in memory BEFORE revealing the canvas
    qrDataUrl.value = await QRCode.toDataURL(info.url, {
      margin: 1,
      width: 240,
      color: {
        dark: '#000000',
        light: '#ffffff',
      },
    });

    // 3. Immediately set state with active data so DOM renders ready content
    status.value = {
      is_active: true,
      url: info.url,
      pin: info.pin,
      target_port: info.target_port,
      local_alive: true,
      paired: false,
      request_count: 0,
      is_public: info.is_public,
    };

    // 4. NOW expand the window and reveal the right canvas with shockwave particles and QR pop-up!
    await invoke('set_window_expanded', { expanded: true }).catch(() => {});
  } catch (err: any) {
    errorMsg.value = typeof err === 'string' ? err : (err?.message || '启动预览失败');
    // Ensure window stays compact on error
    await invoke('set_window_expanded', { expanded: false }).catch(() => {});
  } finally {
    isLaunching.value = false;
  }
}

async function stopPreview() {
  isLaunching.value = true;
  errorMsg.value = '';
  try {
    // 1. Instantly collapse window and hide right canvas
    status.value.is_active = false;
    qrDataUrl.value = '';
    await invoke('set_window_expanded', { expanded: false }).catch(() => {});

    // 2. Clean up tunnel backend
    await invoke('stop_preview');
    await pollStatus();
  } catch (err: any) {
    errorMsg.value = typeof err === 'string' ? err : (err?.message || '停止服务失败');
  } finally {
    isLaunching.value = false;
  }
}

async function pollStatus() {
  try {
    const s = await invoke<PreviewStatus>('get_preview_status');
    status.value = s;
    if (s.is_active && s.url && !qrDataUrl.value) {
      qrDataUrl.value = await QRCode.toDataURL(s.url, {
        margin: 1,
        width: 240,
        color: {
          dark: '#000000',
          light: '#ffffff',
        },
      });
    }
  } catch (err) {
    console.error('Failed to poll status:', err);
  }
}

async function copyUrl() {
  if (!status.value.url) return;
  try {
    await navigator.clipboard.writeText(status.value.url);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch (err) {
    console.error('Failed to copy:', err);
  }
}

onMounted(async () => {
  await scanPorts();
  await pollStatus();
  // Ensure window size matches initial state
  await invoke('set_window_expanded', { expanded: status.value.is_active }).catch(() => {});
  pollTimer = window.setInterval(pollStatus, 1500);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>
