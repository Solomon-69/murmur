import React, { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  ACCENT_COLORS,
  MAIN_ACCENTS,
  MORE_ACCENT_GROUPS,
  isMainAccent,
  normalizeAccent,
} from "@/lib/constants/accents";
import { ColorSwatch } from "./ColorSwatch";

interface AccentPickerProps {
  value: string;
  onChange: (id: string) => void;
}

const MAIN: string[] = [...MAIN_ACCENTS];
/** Rough menu height (three rows plus padding) used to pick a side. */
const MENU_HEIGHT = 190;
const MORE_ROWS = MORE_ACCENT_GROUPS.map((g) => Object.keys(g.colors));
const MORE_ALL = MORE_ROWS.flat();
/** Multicolour wheel for the "More colours" button: hard-edged flat slices, no blending. */
const WHEEL_IDS = [
  "coral",
  "tangerine",
  "ochre",
  "olive",
  "aqua",
  "electric",
  "violet",
  "magenta",
];
const WHEEL = `conic-gradient(${WHEEL_IDS.map(
  (id, i) =>
    `${ACCENT_COLORS[id]} ${((i / WHEEL_IDS.length) * 100).toFixed(2)}% ${(((i + 1) / WHEEL_IDS.length) * 100).toFixed(2)}%`,
).join(", ")})`;

/** Arrow-key target in a set of rows. Left/right walk the whole list,
 *  up/down jump rows and keep the column where the row is long enough. */
function nextId(rows: string[][], current: string, key: string) {
  const all = rows.flat();
  const flat = Math.max(0, all.indexOf(current));
  const row = Math.max(
    0,
    rows.findIndex((r) => r.includes(current)),
  );
  const col = Math.max(0, rows[row].indexOf(current));
  const jump = (d: number) => {
    const r = rows[(row + d + rows.length) % rows.length];
    return r[Math.min(col, r.length - 1)];
  };
  switch (key) {
    case "ArrowRight":
      return all[(flat + 1) % all.length];
    case "ArrowLeft":
      return all[(flat - 1 + all.length) % all.length];
    case "ArrowDown":
      return rows.length > 1 ? jump(1) : null;
    case "ArrowUp":
      return rows.length > 1 ? jump(-1) : null;
    case "Home":
      return all[0];
    case "End":
      return all[all.length - 1];
    default:
      return null;
  }
}

interface SwatchProps {
  id: string;
  selected: boolean;
  focusable: boolean;
  name: string;
  onPick: (id: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLButtonElement>) => void;
  buttonRef: (el: HTMLButtonElement | null) => void;
}

const Swatch: React.FC<SwatchProps> = ({
  id,
  selected,
  focusable,
  name,
  onPick,
  onKeyDown,
  buttonRef,
}) => (
  <ColorSwatch
    ref={buttonRef}
    fill={ACCENT_COLORS[id]}
    selected={selected}
    role="radio"
    aria-checked={selected}
    aria-label={name}
    title={name}
    data-id={id}
    tabIndex={focusable ? 0 : -1}
    onClick={() => onPick(id)}
    onKeyDown={onKeyDown}
  />
);

export const AccentPicker: React.FC<AccentPickerProps> = ({
  value,
  onChange,
}) => {
  const { t } = useTranslation();
  const a = "settings.appearance.accent";
  const selected = normalizeAccent(value);
  const inMore = !isMainAccent(selected);
  const [open, setOpen] = useState(false);
  /** Opens upward when there is no room below the button. */
  const [above, setAbove] = useState(false);
  const openMenu = () => {
    const r = moreRef.current?.getBoundingClientRect();
    setAbove(!!r && r.bottom + MENU_HEIGHT > window.innerHeight);
    setOpen(true);
  };
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const moreRef = useRef<HTMLButtonElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const panelId = useId();

  const close = (refocus: boolean) => {
    setOpen(false);
    if (refocus) moreRef.current?.focus();
  };

  // Outside click closes the menu.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", onDown);
    return () => document.removeEventListener("pointerdown", onDown);
  }, [open]);

  // Opening moves focus to the picked colour (or the first one).
  useEffect(() => {
    if (!open) return;
    const target = inMore ? selected : MORE_ALL[0];
    refs.current[`more-${target}`]?.focus();
  }, [open, inMore, selected]);

  const mainKeys = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    const from = inMore ? MAIN[0] : selected;
    const id = nextId([MAIN], from, e.key);
    if (!id) return;
    e.preventDefault();
    onChange(id);
    refs.current[`main-${id}`]?.focus();
  };

  const moreKeys = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      close(true);
      return;
    }
    const current = e.currentTarget.dataset.id ?? "";
    const id = nextId(MORE_ROWS, current, e.key);
    if (!id) return;
    e.preventDefault();
    refs.current[`more-${id}`]?.focus();
  };

  const pickMore = (id: string) => {
    onChange(id);
    close(true);
  };

  return (
    <div
      ref={wrapRef}
      className="relative mx-auto flex w-fit flex-wrap items-center justify-center gap-x-2.5 gap-y-3 py-1"
      onBlur={(e) => {
        if (open && !wrapRef.current?.contains(e.relatedTarget as Node))
          setOpen(false);
      }}
    >
      <div
        role="radiogroup"
        aria-label={t(`${a}.title`)}
        className="flex items-center gap-2.5"
      >
        {MAIN.map((id) => (
          <Swatch
            key={id}
            id={id}
            name={t(`${a}.${id}`)}
            selected={id === selected}
            focusable={inMore ? id === MAIN[0] : id === selected}
            onPick={onChange}
            onKeyDown={mainKeys}
            buttonRef={(el) => {
              refs.current[`main-${id}`] = el;
            }}
          />
        ))}
      </div>

      <ColorSwatch
        ref={moreRef}
        fill={WHEEL}
        selected={inMore}
        className="ms-1"
        aria-haspopup="true"
        aria-expanded={open}
        aria-controls={panelId}
        onClick={() => (open ? setOpen(false) : openMenu())}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown" && !open) {
            e.preventDefault();
            openMenu();
          } else if (e.key === "Escape" && open) {
            e.preventDefault();
            close(true);
          }
        }}
        aria-label={t(`${a}.more`)}
        title={inMore ? t(`${a}.${selected}`) : t(`${a}.more`)}
      />

      {open && (
        <div
          id={panelId}
          role="radiogroup"
          aria-label={t(`${a}.more`)}
          className={`absolute end-0 z-40 ${above ? "bottom-full mb-2 origin-bottom-right" : "top-full mt-2 origin-top-right"} flex animate-drop flex-col gap-2.5 rounded-[16px] border border-line bg-card p-3.5 shadow-[0_16px_40px_-12px_rgba(15,15,15,0.35),0_2px_6px_rgba(15,15,15,0.08)] motion-reduce:animate-none`}
        >
          {MORE_ACCENT_GROUPS.map((group) => (
            <div
              key={group.id}
              className="grid grid-cols-[3.75rem_auto] items-center gap-3"
            >
              <span className="text-end text-xs font-medium text-ink-2">
                {t(`${a}.groups.${group.id}`)}
              </span>
              <div className="flex items-center gap-2.5">
                {Object.keys(group.colors).map((id) => (
                  <Swatch
                    key={id}
                    id={id}
                    name={t(`${a}.${id}`)}
                    selected={id === selected}
                    focusable={inMore ? id === selected : id === MORE_ALL[0]}
                    onPick={pickMore}
                    onKeyDown={moreKeys}
                    buttonRef={(el) => {
                      refs.current[`more-${id}`] = el;
                    }}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
