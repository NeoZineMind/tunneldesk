<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { CheckCircleOutlined, SaveOutlined } from '@ant-design/icons-vue'
import PasswordField from './PasswordField.vue'
import { portRule, requiredRule } from '@/shared/domain/validators'
import { useAppStore } from '@/stores/appStore'
import { usePasswordStore } from '@/stores/passwordStore'

const store = useAppStore()
const passwordStore = usePasswordStore()
const formRef = ref()
const passwordInput = ref('')
const testFeedback = ref<{ type: 'success' | 'error'; text: string } | null>(null)

const hasSavedPassword = computed(() => passwordStore.hasPassword(store.currentTunnel.id))
const dirty = computed(() => store.currentTunnelDirty)

watch(
  () => store.currentTunnel.id,
  () => {
    passwordInput.value = ''
    testFeedback.value = null
  },
  { immediate: true },
)

async function save() {
  await formRef.value?.validate()
  await store.saveTunnel(passwordInput.value)
  if (store.messageType === 'success') {
    passwordInput.value = ''
  }
}

async function test() {
  await formRef.value?.validate()
  await store.testTunnel(passwordInput.value)
  testFeedback.value = {
    type: store.messageType === 'success' ? 'success' : 'error',
    text: store.message || (store.messageType === 'success' ? 'SSH 连接成功' : 'SSH 连接失败'),
  }
  if (store.messageType === 'success') {
    passwordInput.value = ''
  }
}
</script>

<template>
  <a-card :bordered="false" class="surface-card tunnel-form-card">
    <template #title>
      <div class="card-title">
        <span class="card-title-main">隧道配置</span>
        <a-tag v-if="dirty" color="warning" class="m-0">有未保存更改</a-tag>
      </div>
    </template>
    <a-form ref="formRef" :model="store.currentTunnel" layout="vertical">
      <div class="grid grid-cols-1 gap-x-4 md:grid-cols-2">
        <a-form-item label="名称" name="name" :rules="[requiredRule('请填写隧道名称')]">
          <a-input v-model:value="store.currentTunnel.name" />
        </a-form-item>
        <a-form-item label="状态" name="enabled">
          <div class="flex min-h-9 items-center gap-2">
            <a-switch v-model:checked="store.currentTunnel.enabled" />
            <span class="text-xs text-[var(--text-muted)]">{{ store.currentTunnel.enabled ? '参与 Profile 启动' : '暂不启动' }}</span>
          </div>
        </a-form-item>
        <a-form-item label="认证方式" :name="['ssh', 'authMethod']">
          <a-select v-model:value="store.currentTunnel.ssh.authMethod">
            <a-select-option value="password">密码</a-select-option>
            <a-select-option value="privateKey">私钥</a-select-option>
            <a-select-option value="agent">ssh-agent</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="Host" :name="['ssh', 'host']" :rules="[requiredRule('请填写 SSH Host')]">
          <a-input v-model:value="store.currentTunnel.ssh.host" />
        </a-form-item>
        <a-form-item label="SSH Port" :name="['ssh', 'port']" :rules="[portRule()]">
          <a-input-number v-model:value="store.currentTunnel.ssh.port" class="w-full" :min="1" :max="65535" />
        </a-form-item>
        <a-form-item label="Username" :name="['ssh', 'username']" :rules="[requiredRule('请填写用户名')]">
          <a-input v-model:value="store.currentTunnel.ssh.username" />
        </a-form-item>
        <a-form-item v-if="store.currentTunnel.ssh.authMethod === 'password'" label="SSH 密码">
          <PasswordField
            v-model="passwordInput"
            :has-saved-password="hasSavedPassword"
            :loading="store.loading"
            @clear="store.clearTunnelPassword(store.currentTunnel.id)"
          />
        </a-form-item>
        <a-form-item v-if="store.currentTunnel.ssh.authMethod === 'privateKey'" label="Identity File" :name="['ssh', 'identityFile']">
          <a-input v-model:value="store.currentTunnel.ssh.identityFile" />
        </a-form-item>
      </div>
      <a-alert
        v-if="testFeedback"
        class="mb-4"
        show-icon
        :type="testFeedback.type"
        :message="testFeedback.type === 'success' ? '连接测试通过' : '连接测试失败'"
        :description="testFeedback.text"
        closable
        @close="testFeedback = null"
      />
      <div class="tunnel-action-bar flex flex-wrap items-center justify-between gap-3 border-t border-[var(--line-soft)] bg-[var(--panel-bg)] pt-4">
        <span class="text-xs text-[var(--text-muted)]">隧道 ID：{{ store.currentTunnel.id }}</span>
        <div class="flex items-center gap-2">
          <a-button :loading="store.loading" @click="test">
            <template #icon><CheckCircleOutlined /></template>
            测试
          </a-button>
          <a-button type="primary" :loading="store.loading" :disabled="!dirty && !passwordInput" @click="save">
            <template #icon><SaveOutlined /></template>
            保存
          </a-button>
        </div>
      </div>
    </a-form>
  </a-card>
</template>

<style scoped>
.tunnel-form-card :deep(.ant-card-body) {
  padding-bottom: 0;
}

.tunnel-action-bar {
  position: sticky;
  z-index: 2;
  bottom: 0;
  margin: 0 -20px;
  padding: 14px 20px;
}
</style>
