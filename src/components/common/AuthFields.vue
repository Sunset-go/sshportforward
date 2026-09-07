<script setup lang="ts">
// 认证方式分段控件 + 密码/私钥字段。密码模式单行；私钥模式含路径、选择文件、指纹条、可选私钥密码。
// 私钥内容只在 Rust 端读取，前端仅持有路径与公钥指纹。
import { ref, watch } from 'vue';
import { useSshTunnel } from '../../composables/useSshTunnel';
import { t } from '../../composables/usePrefs';
import type { AuthMethod, KeyInfo } from '../../types';

const props = defineProps<{
  method: AuthMethod;
  password: string;
  keyPath: string;
  passphrase: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:method', v: AuthMethod): void;
  (e: 'update:password', v: string): void;
  (e: 'update:keyPath', v: string): void;
  (e: 'update:passphrase', v: string): void;
}>();

const { pickKeyFilePath, inspectPrivateKey } = useSshTunnel();

const keyInfo = ref<KeyInfo | null>(null);

function setMethod(m: AuthMethod): void {
  if (props.disabled || m === props.method) return;
  emit('update:method', m);
}

async function chooseFile(): Promise<void> {
  const path = await pickKeyFilePath();
  if (path) {
    emit('update:keyPath', path);
    emit('update:method', 'privateKey');
  }
}

async function refreshKeyInfo(): Promise<void> {
  if (props.method === 'privateKey' && props.keyPath.trim()) {
    keyInfo.value = await inspectPrivateKey(props.keyPath.trim());
  } else {
    keyInfo.value = null;
  }
}

// 私钥路径或认证方式变化时刷新指纹（immediate 覆盖编辑已有主机时的初次回显）
watch(() => [props.keyPath, props.method] as const, () => { void refreshKeyInfo(); }, { immediate: true });

function displayKeyType(type: string): string {
  const lower = type.toLowerCase();
  if (lower === 'ssh-ed25519') return 'ED25519';
  if (lower === 'ssh-rsa') return 'RSA';
  if (lower.startsWith('ecdsa-sha2-nistp')) return 'ECDSA P-' + lower.slice('ecdsa-sha2-nistp'.length);
  return type.toUpperCase();
}

function truncateFingerprint(fp: string): string {
  const i = fp.indexOf(':');
  if (i < 0) return fp;
  const prefix = fp.slice(0, i + 1);
  const body = fp.slice(i + 1);
  if (body.length <= 19) return fp;
  return `${prefix}${body.slice(0, 16)}...${body.slice(-3)}`;
}

const keyInfoText = (): string => {
  if (!keyInfo.value) return '';
  return t('auth.keyLoaded', {
    type: displayKeyType(keyInfo.value.keyType),
    fingerprint: truncateFingerprint(keyInfo.value.fingerprint),
  });
};
</script>

<template>
  <div class="auth-fields">
    <!-- 分段控件 -->
    <div class="segment" role="tablist">
      <button
        type="button"
        class="seg"
        :class="{ active: method === 'password' }"
        :disabled="disabled"
        @click="setMethod('password')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <rect x="5" y="11" width="14" height="9" rx="2" stroke="currentColor" stroke-width="1.5" />
          <path d="M8 11V8a4 4 0 0 1 8 0v3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span>{{ t('auth.password') }}</span>
      </button>
      <button
        type="button"
        class="seg"
        :class="{ active: method === 'privateKey' }"
        :disabled="disabled"
        @click="setMethod('privateKey')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="8" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
          <path d="M11 11l9 9M15 19l2-2M18 16l2-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span>{{ t('auth.privateKey') }}</span>
      </button>
    </div>

    <!-- 密码模式 -->
    <div v-if="method === 'password'" class="field">
      <label>{{ t('auth.password') }}</label>
      <input
        class="input pw"
        :type="'password'"
        :value="password"
        :disabled="disabled"
        @input="emit('update:password', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- 私钥模式 -->
    <template v-else>
      <div class="field">
        <label>{{ t('auth.keyFile') }}</label>
        <div class="key-row">
          <div class="path-wrap">
            <svg class="key-ico" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="8" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
              <path d="M11 11l9 9M15 19l2-2M18 16l2-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            <input
              class="input path"
              type="text"
              :value="keyPath"
              :disabled="disabled"
              spellcheck="false"
              @input="emit('update:keyPath', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <button class="browse" type="button" :disabled="disabled" @click="chooseFile">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
            </svg>
            {{ t('auth.chooseFile') }}
          </button>
        </div>
      </div>

      <div v-if="keyInfo" class="key-info">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M5 13l4 4L19 7" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span class="mono">{{ keyInfoText() }}</span>
      </div>

      <div class="field">
        <label>
          {{ t('auth.passphrase') }}
          <span class="hint">{{ t('auth.passphraseHint') }}</span>
        </label>
        <input
          class="input pw"
          :type="'password'"
          :value="passphrase"
          :disabled="disabled"
          @input="emit('update:passphrase', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.auth-fields {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.segment {
  display: inline-flex;
  align-items: center;
  gap: 0;
  width: 176px;
  height: 28px;
  padding: 2px;
  background: var(--input);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}
.seg {
  flex: 1;
  height: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.seg.active {
  background: var(--blue);
  color: #fff;
}
.seg:focus-visible {
  outline: 2px solid var(--blue);
  outline-offset: -2px;
}
.seg:disabled { cursor: not-allowed; opacity: 0.5; }

.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.field label {
  font-size: 12px;
  color: var(--text-2);
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.hint { color: var(--text-3); }

.input {
  height: 30px;
  padding: 0 10px;
  background: var(--input);
  border: 1px solid var(--border);
  border-radius: var(--radius-input);
  color: var(--text);
  font-size: 13px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.input:focus {
  outline: none;
  border-color: var(--blue);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.input:disabled { opacity: 0.45; cursor: not-allowed; }
.input.pw { width: 100%; max-width: 460px; }

.key-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}
.path-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 10px;
  background: var(--input);
  border: 1px solid var(--border);
  border-radius: var(--radius-input);
  transition: border-color 0.15s, box-shadow 0.15s;
}
.path-wrap:focus-within {
  border-color: var(--blue);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.key-ico { color: var(--text-3); flex-shrink: 0; }
.path {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0;
  border: none;
  background: transparent;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
}
.path:focus { outline: none; box-shadow: none; }

.browse {
  flex-shrink: 0;
  width: 96px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  background: var(--btn-bg);
  border: 1px solid var(--btn-border);
  border-radius: var(--radius-btn);
  color: var(--text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.browse:hover { border-color: var(--blue); color: var(--text); }
.browse:disabled { opacity: 0.45; cursor: not-allowed; }
.browse:focus-visible { outline: 2px solid var(--blue); outline-offset: 1px; }

.key-info {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 30px;
  padding: 0 10px;
  background: var(--success-bg);
  border: 1px solid var(--success-border);
  border-radius: var(--radius-input);
  color: var(--green-text);
  font-size: 12px;
}
.mono {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
