<script setup lang="ts">
import { computed, ref } from 'vue';
import { useSshTunnel } from '../composables/useSshTunnel';
import { t } from '../composables/usePrefs';

const { state, currentHost, toggleShowPassword, pickKeyFile, startTunnel, stopTunnel } = useSshTunnel();

const collapsed = ref(false);

const isConnecting = computed(() => state.connState === 'connecting' || state.connState === 'disconnecting');
const isConnected = computed(() => state.connState === 'connected');
const startDisabled = computed(
  () => state.connState === 'connecting' || state.connState === 'connected' || state.connState === 'disconnecting',
);
const stopDisabled = computed(() => state.connState !== 'connected');
</script>

<template>
  <section class="card">
    <div class="card-head" :class="{ collapsed }" @click="collapsed = !collapsed">
      <h2>{{ t('conn.title') }}</h2>
      <span class="sub">{{ t('conn.subtitle') }}</span>
      <div class="spacer"></div>
      <svg class="chevron" :class="{ down: !collapsed }" width="14" height="14" viewBox="0 0 24 24" fill="none">
        <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </div>

    <div v-show="!collapsed">
      <div class="body" v-if="currentHost">
        <div class="form-grid">
        <div class="field">
          <label>{{ t('conn.host') }}</label>
          <input type="text" v-model="currentHost.host" placeholder="192.168.1.100" :disabled="isConnected" />
          <p v-if="state.errors.host" class="err">{{ state.errors.host }}</p>
        </div>

        <div class="field">
          <label>{{ t('conn.port') }}</label>
          <input type="number" min="1" max="65535" v-model.number="currentHost.port" :disabled="isConnected" />
          <p v-if="state.errors.port" class="err">{{ state.errors.port }}</p>
        </div>

        <div class="field">
          <label>{{ t('conn.username') }}</label>
          <input type="text" v-model="currentHost.username" placeholder="Administrator" :disabled="isConnected" />
          <p v-if="state.errors.username" class="err">{{ state.errors.username }}</p>
        </div>

        <div class="field">
          <label>{{ t('conn.password') }}</label>
          <div class="input-wrap">
            <input
              :type="state.showPassword ? 'text' : 'password'"
              v-model="currentHost.password"
              placeholder="••••••••"
              :disabled="isConnected"
            />
            <button class="icon-btn" type="button" @click="toggleShowPassword" :title="state.showPassword ? t('conn.hide') : t('conn.show')" :disabled="isConnected">
              <svg v-if="state.showPassword" width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" stroke="currentColor" stroke-width="2" />
                <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2" />
              </svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M3 3l18 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                <path d="M10.6 6.1A10.9 10.9 0 0 1 12 6c6.5 0 10 6 10 6a17.6 17.6 0 0 1-3.2 4M6.6 6.6A17.6 17.6 0 0 0 2 12s3.5 7 10 7a10.8 10.8 0 0 0 4.4-.9" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2" />
              </svg>
            </button>
          </div>
        </div>

        <div class="field span-2">
          <label>{{ t('conn.keyPath') }}</label>
          <div class="input-wrap">
            <input type="text" v-model="currentHost.keyPath" :placeholder="t('conn.keyPathPlaceholder')" :disabled="isConnected" />
            <button class="browse" type="button" @click="pickKeyFile" :disabled="isConnected">{{ t('conn.browse') }}</button>
          </div>
        </div>
      </div>

      <div class="actions">
        <button class="btn primary" :disabled="startDisabled" type="button" @click="startTunnel">
          <span v-if="isConnecting" class="spinner"></span>
          {{ state.connState === 'connecting' ? t('conn.connecting') : t('conn.start') }}
        </button>
        <button class="btn" :disabled="stopDisabled" type="button" @click="stopTunnel">{{ t('conn.stop') }}</button>
      </div>
    </div>
  </div>
</section>
</template>

<style scoped>
.card {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--panel-shadow);
  overflow: hidden;
}
.card-head {
  display: flex; align-items: center; gap: 10px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  background: linear-gradient(180deg, rgba(47,129,247,0.06), transparent);
  cursor: pointer; user-select: none;
}
.card-head.collapsed { border-bottom-color: transparent; }
.card-head h2 { font-size: 13px; font-weight: 600; color: var(--text); margin: 0; }
.sub { font-size: 12px; color: var(--text-dim); }
.spacer { flex: 1; }
.chevron { color: var(--text-dim); transition: transform 0.2s; flex-shrink: 0; }
.chevron.down { transform: rotate(180deg); }

.body { display: flex; gap: 16px; padding: 14px; align-items: stretch; }
.form-grid {
  flex: 1; min-width: 0;
  display: grid; grid-template-columns: 1fr 1fr; gap: 12px;
}
.field { display: flex; flex-direction: column; gap: 5px; min-width: 0; }
.field.span-2 { grid-column: 1 / -1; }
.field label { font-size: 12px; color: var(--text-dim); }
.field input {
  width: 100%;
  padding: 7px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text); font-size: 13px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.field input:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.field input:disabled { opacity: 0.45; cursor: not-allowed; }
.err { margin: 0; font-size: 11px; color: var(--error); }

.input-wrap { display: flex; gap: 6px; }
.input-wrap input { flex: 1; min-width: 0; }
.icon-btn, .browse {
  flex-shrink: 0;
  display: inline-flex; align-items: center; justify-content: center;
  padding: 0 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text-dim); cursor: pointer;
  transition: all 0.15s;
}
.icon-btn:hover, .browse:hover { border-color: var(--accent); color: var(--accent); }
.icon-btn:disabled, .browse:disabled { opacity: 0.45; cursor: not-allowed; pointer-events: none; }

.actions { display: flex; flex-direction: column; gap: 8px; justify-content: flex-end; min-width: 88px; }
.btn {
  padding: 8px 14px; font-size: 13px;
  background: var(--bg-input); color: var(--text);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.15s;
  display: inline-flex; align-items: center; justify-content: center; gap: 6px;
}
.btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.btn:active:not(:disabled) { transform: translateY(1px); }
.btn:disabled { opacity: 0.45; cursor: not-allowed; }
.btn.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn.primary:hover:not(:disabled) { filter: brightness(1.1); color: #fff; }

.spinner {
  width: 13px; height: 13px;
  border: 2px solid rgba(255,255,255,0.4); border-top-color: #fff;
  border-radius: 50%; animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

@media (max-width: 640px) {
  .body { flex-direction: column; }
  .form-grid { grid-template-columns: 1fr; }
  .actions { flex-direction: row; min-width: 0; }
  .actions .btn { flex: 1; }
}
</style>

