import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface PopoverState {
  version: string;
  busy: boolean;
  show_in_dock: boolean;
  update_checks_enabled: boolean;
}

// Menu bar dropdown labels (English only, like the native tray strings' fallback).
const L = {
  title: "murmur",
  ready: "Ready",
  working: "Working...",
  cancel: "Cancel",
  esc: "esc",
  dock: "Show in Dock",
  copy: "Copy last transcript",
  update: "Update App",
  settings: "Settings...",
  quit: "Quit murmur",
};

type Action = "cancel" | "settings" | "copy_last_transcript" | "quit";

const run = (action: Action) =>
  invoke("tray_popover_action", { action }).catch(() => undefined);

/** Spring switch: flips the moment it is clicked and never snaps back. */
function SmoothSwitch(props: {
  on: boolean;
  label: string;
  onChange: (on: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={props.on}
      className={`pv-row pv-switch-row`}
      onClick={() => props.onChange(!props.on)}
    >
      <span>{props.label}</span>
      <span className={`pv-switch ${props.on ? "on" : ""}`}>
        <span className="pv-knob" />
      </span>
    </button>
  );
}

export default function Popover() {
  const [state, setState] = useState<PopoverState | null>(null);
  const [dock, setDock] = useState(false);
  const [open, setOpen] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const [update, setUpdate] = useState<string>("");

  const refresh = useCallback(async () => {
    try {
      const s = await invoke<PopoverState>("tray_popover_state");
      setState(s);
      setDock(s.show_in_dock);
    } catch {
      // Keep the last state.
    }
  }, []);

  useEffect(() => {
    refresh();
    const subs = [
      listen<number>("popover-open", () => {
        setOpen(false);
        refresh();
        requestAnimationFrame(() => requestAnimationFrame(() => setOpen(true)));
      }),
      listen("popover-close", () => setOpen(false)),
      listen<string>("update-app-progress", (e) => setUpdate(e.payload)),
    ];
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") run("cancel");
    };
    window.addEventListener("keydown", onKey);
    return () => {
      subs.forEach((p) => p.then((u) => u()));
      window.removeEventListener("keydown", onKey);
    };
  }, [refresh]);

  const toggleDock = (on: boolean) => {
    setDock(on); // flip now; the setting follows
    invoke("change_show_in_dock_setting", { enabled: on }).catch(() =>
      setDock(!on),
    );
  };

  const updateApp = async () => {
    setUpdate("Checking for updates...");
    try {
      const r = await invoke<string>("update_app");
      setUpdate(r === "latest" ? "You have the latest version" : r);
    } catch (e) {
      setUpdate(`Update failed: ${String(e)}`);
    }
  };

  // The window takes the panel's real height, so the glass fits exactly.
  useEffect(() => {
    const el = panelRef.current;
    if (!el) return;
    const send = () =>
      invoke("tray_popover_resize", {
        height: Math.ceil(el.getBoundingClientRect().height),
      }).catch(() => {});
    send();
    const ro = new ResizeObserver(send);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const busy = state?.busy ?? false;

  return (
    <div className={`pv-stage ${open ? "open" : ""}`}>
      <div className="pv-panel" ref={panelRef}>
        <div className="pv-head">
          <span className="pv-title">{L.title}</span>
          <span className="pv-version">{state ? `v${state.version}` : ""}</span>
        </div>
        <div className={`pv-status ${busy ? "busy" : ""}`}>
          <span className="pv-dot" />
          {busy ? L.working : L.ready}
        </div>
        <button
          type="button"
          className="pv-row pv-cancel"
          onClick={() => {
            run("cancel");
            setTimeout(refresh, 150);
          }}
        >
          <span>{L.cancel}</span>
          <kbd>{L.esc}</kbd>
        </button>
        <div className="pv-sep" />
        <SmoothSwitch on={dock} label={L.dock} onChange={toggleDock} />
        <div className="pv-sep" />
        <button
          type="button"
          className="pv-row"
          onClick={() => run("copy_last_transcript")}
        >
          <span>{L.copy}</span>
        </button>
        {state?.update_checks_enabled !== false && (
          <button type="button" className="pv-row" onClick={updateApp}>
            <span>{L.update}</span>
          </button>
        )}
        {update && <div className="pv-note">{update}</div>}
        <button
          type="button"
          className="pv-row"
          onClick={() => run("settings")}
        >
          <span>{L.settings}</span>
        </button>
        <div className="pv-sep" />
        <button type="button" className="pv-row" onClick={() => run("quit")}>
          <span>{L.quit}</span>
        </button>
      </div>
    </div>
  );
}
