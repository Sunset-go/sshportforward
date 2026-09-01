<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useSshTunnel } from '../composables/useSshTunnel';
import { t } from '../composables/usePrefs';
import type { RuleType } from '../types';

const { state, editingRule, saveRuleEdit, closeEditModal } = useSshTunnel();

const typeOptions: { value: RuleType; flag: string }[] = [
  { value: 'local', flag: '-L' },
  { value: 'remote', flag: '-R' },
  { value: 'dynamic', flag: '-D' },
];

const formType = ref<RuleType>('local');
const formListen = ref('');
const formTarget = ref('');
const formNote = ref('');

const isDynamic = computed(() => formType.value === 'dynamic');
const listenLabel = computed(() => (formType.value === 'remote' ? t('common.localAddr') : t('common.listenAddr')));
const targetLabel = computed(() => (formType.value === 'remote' ? t('common.remoteAddr') : t('common.targetAddr')));

watch(
  () => state.editingRuleId,
  (id) => {
    if (!id) return;
    const r = editingRule.value;
    if (!r) return;
    formType.value = r.type;
    formListen.value = r.listenAddr;
    formTarget.value = r.targetAddr;
    formNote.value = r.note;
  },
  { immediate: true },
);

function handleSave(): void {
  saveRuleEdit({
    type: formType.value,
    listenAddr: formListen.value,
    targetAddr: formTarget.value,
    note: formNote.value,
  });
}

function onKeydown(e: KeyboardEvent): void {
  if (e.key === 'Escape' && state.editingRuleId) closeEditModal();
}
onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <Transition name="modal">
    <div v-if="state.editingRuleId" class="overlay" @click.self="closeEditModal">
      <div class="modal" role="dialog" aria-modal="true">
        <div class="modal-head">
          <h3>{{ t('rm.title') }}</h3>
          <button class="close-btn" type="button" :title="t('common.close')" @click="closeEditModal">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <path d="M6 6l12 12M18 6L6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <div class="form-row">
            <label>{{ t('rm.type') }}</label>
            <div class="type-group">
              <button
                v-for="opt in typeOptions"
                :key="opt.value"
                type="button"
                class="type-btn"
                :class="{ active: formType === opt.value }"
                @click="formType = opt.value"
              >
                {{ t('common.' + opt.value) }} <span class="flag">{{ opt.flag }}</span>
              </button>
            </div>
          </div>
          <div class="form-row">
            <label>{{ listenLabel }}</label>
            <input type="text" v-model="formListen" placeholder="127.0.0.1:3080" class="text-input" />
          </div>
          <div v-if="!isDynamic" class="form-row">
            <label>{{ targetLabel }}</label>
            <input type="text" v-model="formTarget" placeholder="127.0.0.1:3080" class="text-input" />
          </div>
          <div class="form-row">
            <label>{{ t('common.note') }}</label>
            <input type="text" v-model="formNote" :placeholder="t('rm.notePlaceholder')" class="text-input" />
          </div>
        </div>
        <div class="modal-foot">
          <button class="btn" type="button" @click="closeEditModal">{{ t('common.cancel') }}</button>
          <button class="btn primary" type="button" @click="handleSave">{{ t('common.save') }}</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(2px);
  display: flex; align-items: center; justify-content: center;
  z-index: 1000;
}
.modal {
  width: 420px; max-width: calc(100vw - 32px);
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--modal-shadow);
  overflow: hidden;
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.modal-head h3 { margin: 0; font-size: 14px; font-weight: 600; color: var(--text); }
.close-btn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border-radius: 4px;
  background: transparent; border: none; color: var(--text-dim);
  cursor: pointer; transition: all 0.12s;
}
.close-btn:hover { color: var(--text); background: rgba(255, 255, 255, 0.06); }
.modal-body { padding: 16px; display: flex; flex-direction: column; gap: 14px; }
.form-row { display: flex; flex-direction: column; gap: 6px; }
.form-row label { font-size: 12px; color: var(--text-dim); }
.text-input {
  width: 100%; padding: 8px 10px;
  background: var(--bg-input); border: 1px solid var(--border);
  border-radius: var(--radius); color: var(--text); font-size: 13px;
  font-family: ui-monospace, Consolas, monospace;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.text-input:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.type-group { display: flex; gap: 6px; }
.type-btn {
  flex: 1; padding: 8px 0; font-size: 13px;
  background: var(--bg-input); color: var(--text-dim);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.12s;
  display: inline-flex; align-items: center; justify-content: center; gap: 4px;
}
.type-btn:hover { color: var(--text); border-color: var(--accent); }
.type-btn.active { background: var(--accent); border-color: var(--accent); color: #fff; }
.type-btn .flag { font-size: 11px; opacity: 0.8; }
.modal-foot {
  display: flex; justify-content: flex-end; gap: 8px;
  padding: 12px 16px; border-top: 1px solid var(--border);
}
.btn {
  padding: 7px 16px; font-size: 13px;
  background: var(--bg-input); color: var(--text);
  border: 1px solid var(--border); border-radius: var(--radius);
  cursor: pointer; transition: all 0.12s;
}
.btn:hover { border-color: var(--accent); color: var(--accent); }
.btn.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn.primary:hover { filter: brightness(1.1); color: #fff; }
.modal-enter-active, .modal-leave-active { transition: opacity 0.18s; }
.modal-enter-active .modal, .modal-leave-active .modal { transition: transform 0.18s, opacity 0.18s; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .modal { transform: scale(0.95) translateY(-8px); opacity: 0; }
.modal-leave-to .modal { transform: scale(0.95); opacity: 0; }
</style>
