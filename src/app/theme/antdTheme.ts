import { computed, type Ref } from 'vue'
import { theme } from 'ant-design-vue'
import type { ThemeConfig } from 'ant-design-vue/es/config-provider/context'

export function useAntdTheme(effectiveTheme: Ref<'light' | 'dark'>) {
  return computed<ThemeConfig>(() => ({
    algorithm: effectiveTheme.value === 'dark' ? theme.darkAlgorithm : theme.defaultAlgorithm,
    token: {
      colorPrimary: '#2563eb',
      borderRadius: 8,
      borderRadiusLG: 10,
      fontSize: 14,
      colorBgLayout: effectiveTheme.value === 'dark' ? '#0f1117' : '#f6f7f9',
      colorBgContainer: effectiveTheme.value === 'dark' ? '#171a22' : '#ffffff',
      colorBgElevated: effectiveTheme.value === 'dark' ? '#1d212b' : '#ffffff',
      colorBorder: effectiveTheme.value === 'dark' ? '#303746' : '#dfe3ea',
      colorText: effectiveTheme.value === 'dark' ? '#f5f7fb' : '#172033',
      colorTextSecondary: effectiveTheme.value === 'dark' ? '#c7cfdd' : '#465168',
      colorTextTertiary: effectiveTheme.value === 'dark' ? '#8994a8' : '#6f7a90',
      controlHeight: 36,
      controlHeightLG: 40,
      motionDurationFast: '0.14s',
      motionDurationMid: '0.18s',
    },
    components: {
      Card: {
        borderRadiusLG: 10,
      },
      Table: {
        headerBg: effectiveTheme.value === 'dark' ? '#1d212b' : '#f7f8fa',
        rowHoverBg: effectiveTheme.value === 'dark' ? '#1d212b' : '#f7f9fc',
      },
      Button: { borderRadius: 8 },
      Input: { borderRadius: 8 },
      Select: { borderRadius: 8 },
      Drawer: { borderRadiusLG: 10 },
    },
  }))
}
