import { computed, onBeforeUnmount, watch } from 'vue'
import { App } from 'ant-design-vue'
import { useAppStore } from '@/stores/appStore'

export function useServiceDrawer(isOpen: () => boolean, isDirty: () => boolean, close: () => void) {
  const store = useAppStore()
  const { modal } = App.useApp()
  let confirming = false
  watch(isOpen, (open) => { store.serviceEditorOpen = open })
  onBeforeUnmount(() => { if (isOpen()) store.serviceEditorOpen = false })

  function requestClose() {
    if (!isOpen() || store.loading || confirming) return
    if (!isDirty()) { close(); return }
    confirming = true
    modal.confirm({
      title: '放弃尚未完成的编辑？',
      content: '这些修改还没有添加到服务配置。',
      okText: '放弃编辑', cancelText: '继续编辑', okButtonProps: { danger: true },
      onOk: () => { confirming = false; close() },
      onCancel: () => { confirming = false },
    })
  }
  const drawerOpen = computed({ get: isOpen, set: (value: boolean) => { if (!value) requestClose() } })
  return { drawerOpen, requestClose }
}
