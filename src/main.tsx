import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import { DatabasePreview } from './components/DatabasePreview'
import './i18n' // Initialize i18n before the app
import './index.css'

const isDatabasePreview = new URLSearchParams(window.location.search).get('window') === 'database-preview'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {isDatabasePreview ? <DatabasePreview /> : <App />}
  </React.StrictMode>,
)