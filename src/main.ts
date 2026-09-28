import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './app/router'
import 'ant-design-vue/dist/reset.css'
import './styles/tailwind.css'
import './styles/base.css'

createApp(App).use(createPinia()).use(router).mount('#app')
