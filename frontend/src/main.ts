import './app.css'
import App from './App.svelte'

document.addEventListener('contextmenu', event => event.preventDefault());

const app = new App({
  target: document.getElementById('app') as HTMLDivElement,
})

export default app
