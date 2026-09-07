<script setup lang="ts">
// 主机表单：连接页与新增主机弹窗共用。校验规则见 utils.isHostDraftValid（只此一份），
// 私钥选择与指纹回显封装在 AuthFields，同样只此一份。
import AuthFields from './AuthFields.vue';
import { t } from '../../composables/usePrefs';
import type { HostDraft } from '../../types';

const model = defineModel<HostDraft>({ required: true });

withDefaults(defineProps<{
  variant?: 'connect' | 'dialog';
  disabled?: boolean;
}>(), {
  variant: 'connect',
  disabled: false,
});
</script>

<template>
  <div class="host-form" :class="variant">
    <template v-if="variant === 'dialog'">
      <div class="field full">
        <label>{{ t('hostbar.namePlaceholder') }}</label>
        <input class="input" type="text" v-model="model.name" :disabled="disabled" />
      </div>
      <div class="row">
        <div class="field">
          <label>{{ t('conn.host') }}</label>
          <input class="input" type="text" v-model="model.host" :disabled="disabled" placeholder="192.168.1.100" />
        </div>
        <div class="field port">
          <label>{{ t('conn.port') }}</label>
          <input class="input" type="number" min="1" max="65535" v-model.number="model.port" :disabled="disabled" />
        </div>
      </div>
      <div class="field full">
        <label>{{ t('conn.username') }}</label>
        <input class="input" type="text" v-model="model.username" :disabled="disabled" placeholder="Administrator" />
      </div>
    </template>

    <template v-else>
      <div class="row three">
        <div class="field">
          <label>{{ t('conn.host') }}</label>
          <input class="input" type="text" v-model="model.host" :disabled="disabled" placeholder="192.168.1.100" />
        </div>
        <div class="field port">
          <label>{{ t('conn.port') }}</label>
          <input class="input" type="number" min="1" max="65535" v-model.number="model.port" :disabled="disabled" />
        </div>
        <div class="field">
          <label>{{ t('conn.username') }}</label>
          <input class="input" type="text" v-model="model.username" :disabled="disabled" placeholder="Administrator" />
        </div>
      </div>
    </template>

    <AuthFields
      v-model:method="model.method"
      v-model:password="model.password"
      v-model:keyPath="model.keyPath"
      v-model:passphrase="model.passphrase"
      :disabled="disabled"
    />
  </div>
</template>

<style scoped>
.host-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.row {
  display: flex;
  gap: 16px;
}
.row.three {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 16px;
}
.row .field { flex: 1; min-width: 0; }
.row .field.port { flex: 0 0 96px; }
.row.three .field.port { flex: 0 0 auto; }

.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}
.field.full { width: 100%; }
.field label { font-size: 12px; color: var(--text-2); }

.input {
  width: 100%;
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
</style>
