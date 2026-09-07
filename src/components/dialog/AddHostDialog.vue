<script setup lang="ts">
// 新增主机弹窗：连接信息 / 认证信息两区分组，认证方式用分段控件即时切换。
// 私钥认证走文件路径（选择文件 + 指纹回显 + 可选私钥密码），保存接真实后端 createHost。
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { useSshTunnel } from '../../composables/useSshTunnel';
import { t } from '../../composables/usePrefs';
import type { AuthMethod, KeyInfo } from '../../types';

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ (e: 'update:open', v: boolean): void; (e: 'saved'): void }>();

const { createHost, pickKeyFilePath, inspectPrivateKey } = useSshTunnel();

interface HostForm {
  name: string;
  host: string;
  port: number;
  username: string;
  method: AuthMethod;
  password: string;
  keyPath: string;
  passphrase: string;
}

const rootRef = ref<HTMLElement | null>(null);

const defaults = (): HostForm => ({
  name: '',
  host: '',
  port: 22,
  username: '',
  method: 'password',
  password: '',
  keyPath: '',
  passphrase: '',
});

const form = reactive<HostForm>(defaults());

// 校验：名称可留空（列表展示用），Host/用户名必填，端口 1-65535
const valid = computed(
  () =>
    !!form.host.trim() &&
    !!form.username.trim() &&
    Number.isFinite(form.port) &&
    form.port > 0 &&
    form.port <= 65535,
);

function close(): void {
  emit('update:open', false);
}

async function save(): Promise<void> {
  if (!valid.value) return;
  const isKey = form.method === 'privateKey';
  const ok = await createHost({
    name: form.name.trim() || t('common.unnamed'),
    host: form.host.trim(),
    port: form.port,
    username: form.username.trim(),
    password: isKey ? '' : form.password,
    keyPath: isKey ? form.keyPath.trim() : '',
    passphrase: isKey ? form.passphrase : '',
  });
  if (ok) {
    emit('saved');
    close();
  }
}

// 端口仅允许数字
function onPortInput(e: Event): void {
  const el = e.target as HTMLInputElement;
  const digits = el.value.replace(/\D/g, '').slice(0, 5);
  form.port = digits === '' ? 0 : parseInt(digits, 10);
  el.value = digits;
}

function setMethod(m: AuthMethod): void {
  form.method = m;
}

// —— 私钥文件选择与指纹回显 ——
const keyInfo = ref<KeyInfo | null>(null);
const picking = ref(false);

async function chooseFile(): Promise<void> {
  picking.value = true;
  try {
    const path = await pickKeyFilePath();
    if (path) form.keyPath = path;
  } finally {
    picking.value = false;
  }
}

async function refreshKeyInfo(): Promise<void> {
  if (form.method === 'privateKey' && form.keyPath.trim()) {
    keyInfo.value = await inspectPrivateKey(form.keyPath.trim());
  } else {
    keyInfo.value = null;
  }
}

watch(() => [form.keyPath, form.method] as const, () => { void refreshKeyInfo(); }, { immediate: true });

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

const keyInfoText = computed(() => {
  if (!keyInfo.value) return '';
  return t('auth.keyLoaded', {
    type: displayKeyType(keyInfo.value.keyType),
    fingerprint: truncateFingerprint(keyInfo.value.fingerprint),
  });
});

// 打开时重置为默认值并聚焦第一个输入框
watch(
  () => props.open,
  (v) => {
    if (v) {
      Object.assign(form, defaults());
      void nextTick(() => rootRef.value?.querySelector('input')?.focus());
    }
  },
);

// Esc 关闭（文档级监听，无论焦点在哪都生效）
function onKeydown(e: KeyboardEvent): void {
  if (props.open && e.key === 'Escape') close();
}
onMounted(() => document.addEventListener('keydown', onKeydown));
onUnmounted(() => document.removeEventListener('keydown', onKeydown));
</script>

<template>
  <Transition name="modal">
    <div v-if="open" class="overlay" @click.self="close">
      <div
        ref="rootRef"
        class="dialog"
        role="dialog"
        aria-modal="true"
        :aria-label="t('dialog.addHostTitle')"
        tabindex="-1"
      >
        <!-- 头部 -->
        <header class="head">
          <div class="head-text">
            <h3>{{ t('dialog.addHostTitle') }}</h3>
            <p>{{ t('dialog.addHostSubtitle') }}</p>
          </div>
          <button class="close" type="button" :title="t('common.close')" @click="close">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M6 6l12 12M18 6L6 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <!-- 主体 -->
        <div class="body">
          <!-- 连接信息 -->
          <section class="group">
            <div class="group-title">{{ t('dialog.groupConnect') }}</div>

            <div class="field">
              <label class="lbl">
                {{ t('hostbar.namePlaceholder') }}
                <span class="hint">{{ t('dialog.nameHint') }}</span>
              </label>
              <input
                v-model="form.name"
                class="ctrl"
                type="text"
                :placeholder="t('dialog.namePlaceholder')"
              />
            </div>

            <div class="row">
              <div class="field grow">
                <label class="lbl">{{ t('conn.host') }}</label>
                <input
                  v-model="form.host"
                  class="ctrl"
                  type="text"
                  :placeholder="t('dialog.hostPlaceholder')"
                />
              </div>
              <div class="field port">
                <label class="lbl">{{ t('conn.port') }}</label>
                <input
                  :value="form.port || ''"
                  class="ctrl"
                  type="text"
                  inputmode="numeric"
                  placeholder="22"
                  @input="onPortInput"
                />
              </div>
            </div>
          </section>

          <!-- 认证信息 -->
          <section class="group">
            <div class="group-title">{{ t('auth.method') }}</div>

            <div class="row">
              <div class="field grow">
                <label class="lbl">{{ t('conn.username') }}</label>
                <input
                  v-model="form.username"
                  class="ctrl"
                  type="text"
                  :placeholder="t('dialog.usernamePlaceholder')"
                />
              </div>
              <div class="field auth">
                <label class="lbl">{{ t('auth.method') }}</label>
                <div class="segment" role="tablist" :aria-label="t('auth.method')">
                  <button
                    type="button"
                    role="tab"
                    class="seg"
                    :class="{ active: form.method === 'password' }"
                    :aria-selected="form.method === 'password'"
                    @click="setMethod('password')"
                  >
                    {{ t('auth.password') }}
                  </button>
                  <button
                    type="button"
                    role="tab"
                    class="seg"
                    :class="{ active: form.method === 'privateKey' }"
                    :aria-selected="form.method === 'privateKey'"
                    @click="setMethod('privateKey')"
                  >
                    {{ t('auth.privateKey') }}
                  </button>
                </div>
              </div>
            </div>

            <!-- 密码模式 -->
            <div v-if="form.method === 'password'" class="field">
              <label class="lbl">{{ t('auth.password') }}</label>
              <input
                v-model="form.password"
                class="ctrl"
                type="password"
                :placeholder="t('auth.passwordPlaceholder')"
              />
            </div>

            <!-- 私钥模式：文件路径 + 选择文件 + 指纹回显 + 可选私钥密码 -->
            <template v-else>
              <div class="field">
                <label class="lbl">{{ t('auth.keyFile') }}</label>
                <div class="key-row">
                  <div class="path-wrap">
                    <svg class="key-ico" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                      <circle cx="8" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
                      <path d="M11 11l9 9M15 19l2-2M18 16l2-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                    </svg>
                    <input
                      :value="form.keyPath"
                      class="path"
                      type="text"
                      spellcheck="false"
                      @input="form.keyPath = ($event.target as HTMLInputElement).value"
                    />
                  </div>
                  <button class="browse" type="button" :disabled="picking" @click="chooseFile">
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
                <span class="mono">{{ keyInfoText }}</span>
              </div>

              <div class="field">
                <label class="lbl">
                  {{ t('auth.passphrase') }}
                  <span class="hint">{{ t('auth.passphraseHint') }}</span>
                </label>
                <input
                  v-model="form.passphrase"
                  class="ctrl"
                  type="password"
                />
              </div>
            </template>
          </section>
        </div>

        <!-- 底部 -->
        <footer class="foot">
          <span class="esc"><kbd>Esc</kbd> {{ t('dialog.escHint') }}</span>
          <div class="actions">
            <button class="btn ghost" type="button" @click="close">{{ t('common.cancel') }}</button>
            <button class="btn primary" type="button" :disabled="!valid" @click="save">{{ t('common.save') }}</button>
          </div>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* 规格设计 token（深色主题，本组件自成体系） */
.dialog {
  --panel: #171b22;
  --panel-2: #1d222b;
  --line: #272d38;
  --line-strong: #333b48;
  --text: #e6eaf2;
  --text-2: #9aa4b6;
  --text-3: #6b7484;
  --placeholder: #5a6373;
  --primary: #2f6feb;
  --primary-hover: #3b7cf5;
  --focus-glow: rgba(47, 111, 235, 0.18);
  --r-ctrl: 10px;
  --r-dialog: 14px;
  --r-btn: 8px;
  --h-input: 36px;
  --h-btn: 34px;
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(7, 9, 12, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}

.dialog {
  width: 520px;
  max-width: calc(100vw - 32px);
  max-height: calc(100vh - 48px);
  display: flex;
  flex-direction: column;
  background: var(--panel);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-dialog);
  box-shadow: 0 16px 56px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  color: var(--text);
}
.dialog:focus { outline: none; }

/* 头部 */
.head {
  flex: 0 0 auto;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 18px 22px 16px;
  border-bottom: 1px solid var(--line);
}
.head-text h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: 0.2px;
}
.head-text p {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--text-3);
}
.close {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-3);
  cursor: pointer;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
}
.close:hover {
  background: var(--panel-2);
  border-color: var(--line);
  color: var(--text);
}
.close:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }

/* 主体 */
.body {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  padding: 20px 22px;
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.group {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.group-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 1px;
  color: var(--text-3);
  text-transform: uppercase;
}

.row {
  display: flex;
  gap: 12px;
  align-items: stretch;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 7px;
  min-width: 0;
}
.field.grow { flex: 1 1 0; }
.field.port { flex: 0 0 132px; }
.field.auth { flex: 0 0 auto; }

.lbl {
  font-size: 12px;
  color: var(--text-2);
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.hint {
  font-size: 11px;
  color: var(--text-3);
  font-weight: 400;
}

.ctrl {
  height: var(--h-input);
  width: 100%;
  padding: 0 12px;
  background: var(--panel-2);
  border: 1px solid var(--line);
  border-radius: var(--r-ctrl);
  color: var(--text);
  font-size: 13px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.ctrl::placeholder { color: var(--placeholder); }
.ctrl:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--focus-glow);
}

/* 私钥文件行 */
.key-row {
  display: flex;
  gap: 10px;
  align-items: stretch;
}
.path-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  height: var(--h-input);
  padding: 0 12px;
  background: var(--panel-2);
  border: 1px solid var(--line);
  border-radius: var(--r-ctrl);
  transition: border-color 0.15s, box-shadow 0.15s;
}
.path-wrap:focus-within {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--focus-glow);
}
.key-ico { color: var(--text-3); flex-shrink: 0; }
.path {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--text);
  font-size: 12.5px;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
}
.path:focus { outline: none; }
.path::placeholder { color: var(--placeholder); }

.browse {
  flex-shrink: 0;
  width: 100px;
  height: var(--h-input);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: var(--panel-2);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-ctrl);
  color: var(--text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.browse:hover { border-color: var(--primary); color: var(--text); }
.browse:disabled { opacity: 0.5; cursor: not-allowed; }
.browse:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }

/* 指纹信息条 */
.key-info {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 30px;
  padding: 0 12px;
  background: var(--success-bg);
  border: 1px solid var(--success-border);
  border-radius: var(--r-ctrl);
  color: var(--green-text);
  font-size: 12px;
}
.mono {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 分段控件 */
.segment {
  display: inline-flex;
  padding: 3px;
  background: var(--panel-2);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-ctrl);
  gap: 0;
}
.seg {
  min-width: 64px;
  height: 28px;
  padding: 0 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 7px;
  color: var(--text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.seg:hover { color: var(--text); }
.seg.active {
  background: var(--primary);
  color: #fff;
}
.seg:focus-visible { outline: 2px solid var(--primary); outline-offset: -2px; }

/* 底部 */
.foot {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 22px;
  background: rgba(0, 0, 0, 0.18);
  border-top: 1px solid var(--line);
}
.esc {
  font-size: 11.5px;
  color: var(--text-3);
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
kbd {
  display: inline-flex;
  align-items: center;
  padding: 1px 6px;
  min-width: 22px;
  height: 18px;
  background: var(--panel-2);
  border: 1px solid var(--line-strong);
  border-bottom-width: 2px;
  border-radius: 4px;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 11px;
  color: var(--text-2);
}
.actions {
  display: flex;
  gap: 10px;
}
.btn {
  height: var(--h-btn);
  min-width: 80px;
  padding: 0 16px;
  border-radius: var(--r-btn);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s, filter 0.12s, opacity 0.12s;
}
.btn:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }
.btn.ghost {
  background: transparent;
  border: 1px solid var(--line-strong);
  color: var(--text-2);
}
.btn.ghost:hover {
  background: var(--panel-2);
  color: var(--text);
}
.btn.primary {
  background: var(--primary);
  border: 1px solid var(--primary);
  color: #fff;
}
.btn.primary:hover:not(:disabled) {
  background: var(--primary-hover);
  border-color: var(--primary-hover);
}
.btn.primary:disabled { opacity: 0.45; cursor: not-allowed; }

/* 过渡 */
.modal-enter-active,
.modal-leave-active { transition: opacity 0.18s; }
.modal-enter-active .dialog,
.modal-leave-active .dialog { transition: transform 0.18s, opacity 0.18s; }
.modal-enter-from,
.modal-leave-to { opacity: 0; }
.modal-enter-from .dialog { transform: scale(0.96) translateY(-6px); opacity: 0; }
.modal-leave-to .dialog { transform: scale(0.97); opacity: 0; }
</style>
