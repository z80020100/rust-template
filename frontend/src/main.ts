import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./style.css";

const MAX_MESSAGES = 100; // Demo limit
const btn = document.getElementById("toggle")!;
const pLog = document.getElementById("producer-log")!;
const cLog = document.getElementById("consumer-log")!;
const countEl = document.getElementById("count")!;
const cleared = new Set<HTMLElement>();
let running = false;

function ts(): string {
  const d = new Date();
  return [d.getHours(), d.getMinutes(), d.getSeconds()]
    .map((v) => String(v).padStart(2, "0"))
    .join(":");
}

function append(log: HTMLElement, text: string): void {
  if (!cleared.has(log)) {
    log.innerHTML = "";
    cleared.add(log);
  }
  const el = document.createElement("div");
  el.className = "msg";
  el.innerHTML = `<span class="ts">${ts()}</span><span class="val">${text}</span>`;
  log.appendChild(el);
  if (log.children.length > MAX_MESSAGES) log.firstChild!.remove();
  log.scrollTop = log.scrollHeight;
}

listen("produce", (e) => {
  append(pLog, `Produce: ${e.payload}`);
});

listen("consume", (e) => {
  append(cLog, `Consume: ${e.payload}`);
  countEl.textContent = `${e.payload} messages`;
});

async function start(): Promise<void> {
  await invoke("start");
  running = true;
  btn.textContent = "Stop";
  btn.classList.add("running");
  document.getElementById("dot")!.classList.add("active");
  document.getElementById("status")!.textContent = "Running";
}

async function stop(): Promise<void> {
  await invoke("stop");
  running = false;
  btn.textContent = "Start";
  btn.classList.remove("running");
  document.getElementById("dot")!.classList.remove("active");
  document.getElementById("status")!.textContent = "Stopped";
}

btn.addEventListener("click", () => (running ? stop() : start()));
