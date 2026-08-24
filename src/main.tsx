import React from 'react'
import ReactDOM from 'react-dom/client'
import { getCurrentWindow } from '@tauri-apps/api/window'
import App from './App.tsx'
import { DatabasePreview } from './components/DatabasePreview'
import './i18n' // Initialize i18n before the app
import './index.css'

async function bootstrap() {
  const isDatabasePreview = getCurrentWindow().label === 'database-preview'

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      {isDatabasePreview ? <DatabasePreview /> : <App />}
    </React.StrictMode>,
  )
}

void bootstrap()