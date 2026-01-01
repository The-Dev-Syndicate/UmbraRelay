import { createApp } from "vue";
import App from "./App.vue";
import "./styles/main.scss";

try {
  const app = createApp(App);
  app.mount("#app");
  console.log("UmbraRelay Vue app mounted successfully");
} catch (error) {
  console.error("Failed to mount Vue app:", error);
  document.body.innerHTML = `
    <div style="padding: 40px; font-family: system-ui; color: #d32f2f;">
      <h1>Error Loading UmbraRelay</h1>
      <p>${error}</p>
      <p>Check the console for more details.</p>
    </div>
  `;
}
