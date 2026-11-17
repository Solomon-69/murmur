import assert from "node:assert/strict";
import { formatKeyCombination, getKeyName, keycapLabel } from "./keyboard";

const keyboardEvent = (value: { code?: string; key?: string }): KeyboardEvent =>
  value as KeyboardEvent;

const compoundKeys = [
  ["ScrollLock", "scrolllock", "Scroll Lock"],
  ["CapsLock", "capslock", "Caps Lock"],
  ["NumLock", "numlock", "Num Lock"],
  ["PageUp", "pageup", "Page Up"],
  ["PageDown", "pagedown", "Page Down"],
  ["PrintScreen", "printscreen", "Print Screen"],
] as const;

for (const [code, stored, displayed] of compoundKeys) {
  assert.equal(getKeyName(keyboardEvent({ code })), stored);
  assert.equal(formatKeyCombination(stored, "linux"), displayed);
}

assert.equal(getKeyName(keyboardEvent({ key: "CapsLock" })), "capslock");
assert.equal(
  getKeyName(keyboardEvent({ code: "AudioVolumeUp" })),
  "audiovolumeup",
);

// Each system prints its own key names.
assert.equal(formatKeyCombination("super+alt+space", "windows"), "Win + Alt + Space");
assert.equal(formatKeyCombination("option+space", "windows"), "Alt + Space");
assert.equal(formatKeyCombination("super+space", "linux"), "Super + Space");
assert.equal(formatKeyCombination("alt+space", "macos"), "Option + Space");
assert.equal(formatKeyCombination("super+space", "macos"), "Command + Space");
assert.equal(keycapLabel("ctrl+win+space", "windows"), "Ctrl + Win + Space");
assert.equal(keycapLabel("option+space", "macos"), "⌥ Space");

console.log("keyboard: all assertions passed");
