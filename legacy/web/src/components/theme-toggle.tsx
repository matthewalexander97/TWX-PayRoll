"use client";

import { useSyncExternalStore } from "react";

const storageKey = "twx-theme";

function snapshot() {
  return document.documentElement.dataset.theme === "dark" ? "dark" : "light";
}

function subscribe(callback: () => void) {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const sync = () => {
    let preference: string | null = null;
    try { preference = localStorage.getItem(storageKey); } catch { /* Theme remains usable without storage. */ }
    document.documentElement.dataset.theme = preference === "light" || preference === "dark"
      ? preference : media.matches ? "dark" : "light";
    callback();
  };
  window.addEventListener("twx-theme-change", callback);
  window.addEventListener("storage", sync);
  media.addEventListener("change", sync);
  return () => {
    window.removeEventListener("twx-theme-change", callback);
    window.removeEventListener("storage", sync);
    media.removeEventListener("change", sync);
  };
}

export function ThemeToggle() {
  const theme = useSyncExternalStore(subscribe, snapshot, () => "light");
  const next = theme === "dark" ? "light" : "dark";
  return (
    <button className="button button-secondary theme-toggle" type="button" aria-label={`Switch to ${next} mode`} onClick={() => {
      document.documentElement.dataset.theme = next;
      try { localStorage.setItem(storageKey, next); } catch { /* Keep the current preference in memory. */ }
      window.dispatchEvent(new Event("twx-theme-change"));
    }}>
      <span aria-hidden="true">{theme === "dark" ? "☀" : "☾"}</span>
      <span>{next === "dark" ? "Dark mode" : "Light mode"}</span>
    </button>
  );
}
