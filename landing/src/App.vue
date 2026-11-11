<template>
  <div class="min-h-screen w-full overflow-x-hidden bg-[#08080a] text-zinc-100 font-sans selection:bg-[#f38020]/20 selection:text-[#f38020]">
    <div class="fixed top-0 left-1/2 -translate-x-1/2 w-[800px] max-w-full h-[360px] bg-gradient-to-b from-[#f38020]/10 to-transparent blur-[120px] pointer-events-none"></div>

    <header class="sticky top-0 z-50 backdrop-blur-md bg-[#08080a]/75 border-b border-white/[0.06]">
      <div class="max-w-6xl mx-auto px-4 sm:px-6 h-14 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-2.5 h-2.5 rounded-full bg-[#f38020] shadow-[0_0_10px_rgba(243,128,32,0.8)]"></div>
          <span class="font-semibold text-sm tracking-tight text-white">FlareBeam</span>
        </div>
        <div class="flex items-center gap-2">
          <a
            :href="REPO_URL"
            aria-label="GitHub"
            class="w-8 h-8 rounded-full flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/[0.06] transition-colors">
            <Github class="w-4 h-4" />
          </a>
          <a
            :href="DOWNLOAD_URL"
            class="h-8 px-3.5 rounded-full bg-white text-zinc-950 font-medium text-xs flex items-center gap-1.5 hover:bg-zinc-200 transition-colors">
            <Download class="w-3.5 h-3.5" />
            下载 macOS 版
          </a>
        </div>
      </div>
    </header>

    <main class="relative z-10">
      <section class="pt-20 sm:pt-28 pb-14 sm:pb-20 px-4 sm:px-6 text-center">
        <h1 class="text-4xl sm:text-6xl md:text-7xl font-bold tracking-tight text-white leading-[1.08]">
          本地服务，<br />一键直达手机。
        </h1>
        <a
          :href="DOWNLOAD_URL"
          class="mt-10 inline-flex h-11 px-5 rounded-xl bg-[#f38020] hover:bg-[#e06f15] text-white font-medium text-xs tracking-wide items-center gap-2 transition shadow-lg shadow-orange-950/30">
          <Download class="w-4 h-4" />
          下载 macOS 版
        </a>
      </section>

      <!-- Simulator: mirrors the desktop app's UI, keep labels in sync with flarebeam/src/App.vue -->
      <section id="simulator" class="pb-20 sm:pb-28 px-4 flex justify-center">
        <div class="relative p-1.5 sm:p-2 rounded-2xl bg-gradient-to-b from-white/10 via-white/5 to-transparent shadow-2xl w-full max-w-[376px] sm:w-auto sm:max-w-none">
          <div
            class="bg-[#09090b] rounded-xl border border-white/[0.08] shadow-2xl overflow-hidden flex flex-col sm:h-[520px] sm:max-w-[calc(100vw-48px)] transition-[width] duration-500 ease-[cubic-bezier(0.16,1,0.3,1)]"
            :class="isSimActive ? 'sm:w-[760px]' : 'sm:w-[360px]'">

            <div class="h-10 border-b border-white/[0.06] px-4 flex items-center justify-between bg-[#0c0c0e]/80 shrink-0">
              <div class="flex items-center gap-2">
                <div class="flex gap-1.5">
                  <span class="w-2.5 h-2.5 rounded-full bg-[#ff5f56]/80"></span>
                  <span class="w-2.5 h-2.5 rounded-full bg-[#ffbd2e]/80"></span>
                  <span class="w-2.5 h-2.5 rounded-full bg-[#27c93f]/80"></span>
                </div>
                <span class="text-[11px] font-medium text-zinc-400 ml-2">FlareBeam</span>
              </div>

              <div v-if="isSimActive" class="flex items-center gap-2 text-[10px] font-mono text-zinc-400">
                <span v-if="isSimPaired" class="text-emerald-400 flex items-center gap-1">
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  手机已连通
                </span>
                <span v-else class="text-amber-400/90 flex items-center gap-1">
                  <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse"></span>
                  等待扫码
                </span>
                <span class="text-zinc-600">·</span>
                <span>{{ simRequests }} 次请求</span>
              </div>
            </div>

            <!-- Side by side on desktop, stacked on phones -->
            <div class="flex-1 flex flex-col sm:flex-row overflow-hidden">
              <div
                class="w-full sm:w-[360px] p-6 flex flex-col justify-between gap-6 shrink-0 bg-[#0c0c0e] overflow-hidden z-10"
                :class="isSimActive ? 'border-b sm:border-b-0 sm:border-r border-white/[0.06]' : ''">

                <div class="space-y-6">
                  <div class="space-y-2.5">
                    <div class="flex items-center justify-between">
                      <label class="text-[11px] font-medium text-zinc-400 tracking-wider">本地端口</label>
                      <button
                        @click="refreshSimPorts"
                        class="text-zinc-500 hover:text-zinc-300 transition p-1"
                        aria-label="刷新端口">
                        <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isSimScanning }" />
                      </button>
                    </div>

                    <div class="flex items-center bg-[#151518] border border-white/[0.08] focus-within:border-white/20 rounded-lg px-3.5 py-2.5 transition">
                      <span class="text-xs font-mono text-zinc-500 mr-2 select-none">localhost:</span>
                      <input
                        v-model.number="simPort"
                        type="number"
                        :disabled="isSimActive"
                        class="w-full bg-transparent text-sm font-mono text-white outline-none placeholder-zinc-600 disabled:opacity-40" />
                    </div>

                    <div class="flex flex-wrap gap-2 pt-0.5">
                      <button
                        v-for="p in SIM_PORTS"
                        :key="p.port"
                        @click="simPort = p.port"
                        :disabled="isSimActive"
                        class="text-xs font-mono px-3 py-1.5 rounded-md border transition-all flex items-center gap-1.5"
                        :class="simPort === p.port
                          ? 'bg-white/10 border-white/20 text-white font-medium shadow-sm'
                          : 'bg-[#151518]/70 border-white/[0.06] text-zinc-400 hover:text-zinc-200 hover:border-white/10'">
                        <span>:{{ p.port }}</span>
                        <span class="text-[10px] text-zinc-500 font-sans">{{ p.name }}</span>
                      </button>
                    </div>
                  </div>

                  <div class="space-y-2.5">
                    <label class="text-[11px] font-medium text-zinc-400 tracking-wider">访问权限</label>
                    <div class="grid grid-cols-2 gap-1 p-1 bg-[#151518] border border-white/[0.08] rounded-lg">
                      <button
                        type="button"
                        @click="simIsPublic = false"
                        :disabled="isSimActive"
                        class="py-2 px-3 rounded-md text-xs font-medium transition-all flex items-center justify-center gap-1.5"
                        :class="!simIsPublic ? 'bg-[#222226] text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-200'">
                        <Lock class="w-3.5 h-3.5 text-amber-500" />
                        <span>PIN 保护</span>
                      </button>
                      <button
                        type="button"
                        @click="simIsPublic = true"
                        :disabled="isSimActive"
                        class="py-2 px-3 rounded-md text-xs font-medium transition-all flex items-center justify-center gap-1.5"
                        :class="simIsPublic ? 'bg-[#222226] text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-200'">
                        <Globe class="w-3.5 h-3.5 text-zinc-300" />
                        <span>公开访问</span>
                      </button>
                    </div>
                  </div>
                </div>

                <div class="pt-4 border-t border-white/[0.06]">
                  <button
                    v-if="!isSimActive"
                    @click="toggleSim"
                    :disabled="isSimLaunching"
                    class="w-full h-11 rounded-lg bg-[#f38020] hover:bg-[#e06f15] text-white font-medium text-xs tracking-wide transition-all flex items-center justify-center gap-2 shadow-sm active:scale-[0.99] disabled:opacity-40">
                    <RefreshCw v-if="isSimLaunching" class="w-4 h-4 animate-spin text-white/90" />
                    <span>{{ isSimLaunching ? '正在连接边缘网络...' : '开启预览' }}</span>
                  </button>
                  <button
                    v-else
                    @click="toggleSim"
                    class="w-full h-11 rounded-lg bg-[#18181b] hover:bg-[#202024] border border-white/10 text-zinc-300 hover:text-red-400 font-medium text-xs tracking-wide transition-all flex items-center justify-center gap-2">
                    <Power class="w-4 h-4" />
                    <span>停止预览</span>
                  </button>
                </div>
              </div>

              <div
                class="flex-1 flex-col items-center justify-center bg-[#09090b] relative overflow-hidden transition-all duration-500 ease-[cubic-bezier(0.16,1,0.3,1)]"
                :class="isSimActive ? 'flex opacity-100 p-8 min-h-[400px] sm:min-h-0' : 'hidden sm:flex sm:w-0 sm:max-w-0 opacity-0 pointer-events-none'">

                <BeamCanvas :active="isSimActive" :paired="isSimPaired" />

                <div v-if="isSimActive" class="w-full max-w-sm flex flex-col items-center space-y-5 z-10 animate-spring-pop">
                  <div class="p-3.5 bg-white rounded-2xl shadow-2xl relative shadow-orange-950/30 ring-1 ring-white/20">
                    <img v-if="simQrUrl" :src="simQrUrl" alt="预览地址二维码" class="w-44 h-44 block" />
                    <div v-if="isSimPaired" class="absolute -bottom-3 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full bg-emerald-500 text-zinc-950 text-[10px] font-semibold tracking-wide shadow-lg whitespace-nowrap flex items-center gap-1.5 animate-bounce">
                      <span class="w-1.5 h-1.5 rounded-full bg-zinc-950"></span>
                      手机已连通
                    </div>
                  </div>

                  <div v-if="!simIsPublic" class="flex flex-col items-center space-y-1.5 pt-0.5">
                    <span class="text-[10px] font-mono uppercase tracking-widest text-zinc-500 font-medium">验证 PIN 码</span>
                    <div class="flex gap-2">
                      <div
                        v-for="(digit, idx) in simPin.split('')"
                        :key="idx"
                        :style="{ animationDelay: `${idx * 60 + 200}ms` }"
                        class="w-9 h-11 rounded-lg bg-[#141416] border border-white/[0.08] text-[#f38020] font-mono text-xl font-bold flex items-center justify-center shadow-inner animate-digit-flip">
                        {{ digit }}
                      </div>
                    </div>
                  </div>

                  <div class="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-[#141416] border border-white/[0.08]">
                    <span class="flex-1 text-[11px] font-mono text-zinc-400 truncate select-all px-1">{{ SIM_URL }}</span>
                    <button
                      @click="simulateScan"
                      class="px-2 py-1 rounded bg-[#222226] hover:bg-white/10 text-[10px] font-mono text-zinc-300 hover:text-white transition">
                      {{ isSimPaired ? '已连通' : '模拟扫码' }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section id="security" class="py-20 sm:py-24 px-4 sm:px-6 border-t border-white/[0.06]">
        <div class="max-w-5xl mx-auto">
          <FailClosedDiagram :port="simPort" />
        </div>
      </section>

      <section class="py-20 sm:py-24 px-6 border-t border-white/[0.06]">
        <ul class="max-w-5xl mx-auto grid grid-cols-1 sm:grid-cols-3 gap-10 sm:gap-8">
          <li v-for="f in FEATURES" :key="f.title">
            <component :is="f.icon" class="w-5 h-5 text-[#f38020] stroke-[1.5] mb-4" />
            <h3 class="text-sm font-semibold text-white mb-1.5">{{ f.title }}</h3>
            <p class="text-sm text-zinc-400 leading-relaxed">{{ f.body }}</p>
          </li>
        </ul>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import QRCode from 'qrcode';
import { Download, Github, RefreshCw, Lock, Globe, Power, Smartphone, KeyRound } from 'lucide-vue-next';
import BeamCanvas from './components/BeamCanvas.vue';
import FailClosedDiagram from './components/FailClosedDiagram.vue';

const REPO_URL = 'https://github.com/edison-land/flarebeam';
const DOWNLOAD_URL = `${REPO_URL}/releases/latest`;

const FEATURES = [
  { icon: Smartphone, title: '手机不用装 App', body: '用相机扫码就能打开，不用配置 VPN 或证书。' },
  { icon: Globe, title: '换什么网络都能连', body: '经 Cloudflare 隧道访问，手机用 Wi-Fi 或流量都可以。' },
  { icon: KeyRound, title: '链接泄露也进不来', body: '二维码里只有地址，PIN 只显示在你的电脑上。' },
];

// Simulator
const SIM_URL = 'https://flare-beam-demo.trycloudflare.com';
const SIM_PORTS = [
  { port: 3000, name: 'Node' },
  { port: 5173, name: 'Vite' },
  { port: 8000, name: 'Python' },
];

const simPort = ref(3000);
const simIsPublic = ref(false);
const isSimActive = ref(false);
const isSimLaunching = ref(false);
const isSimPaired = ref(false);
const isSimScanning = ref(false);
const simRequests = ref(0);
const simPin = ref('5728');
const simQrUrl = ref('');

function refreshSimPorts() {
  isSimScanning.value = true;
  setTimeout(() => (isSimScanning.value = false), 400);
}

function toggleSim() {
  if (isSimActive.value) {
    isSimActive.value = false;
    isSimPaired.value = false;
    simRequests.value = 0;
    return;
  }
  isSimLaunching.value = true;
  simPin.value = Math.floor(1000 + Math.random() * 9000).toString();
  setTimeout(() => {
    isSimLaunching.value = false;
    isSimActive.value = true;
  }, 600);
}

function simulateScan() {
  isSimPaired.value = true;
  simRequests.value += 3;
}

onMounted(async () => {
  simQrUrl.value = await QRCode.toDataURL(SIM_URL, { margin: 1, width: 240 });
});
</script>
