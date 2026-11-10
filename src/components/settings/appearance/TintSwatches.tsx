import React from "react";
import { ColorSwatch } from "./ColorSwatch";

interface TintSwatchesProps {
  ids: string[];
  value: string;
  label: string;
  /** Visible name for each id. */
  name: (id: string) => string;
  /** CSS background for each id. */
  fill: (id: string) => string;
  onChange: (id: string) => void;
  ring?: "light" | "dark";
}

/** A row of shared colour circles. Arrow keys move the choice. */
export const TintSwatches: React.FC<TintSwatchesProps> = ({
  ids,
  value,
  label,
  name,
  fill,
  onChange,
  ring = "light",
}) => {
  const refs = React.useRef<Record<string, HTMLButtonElement | null>>({});
  const current = ids.includes(value) ? value : ids[0];
  const onKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    const i = Math.max(0, ids.indexOf(current));
    const step =
      e.key === "ArrowRight" || e.key === "ArrowDown"
        ? 1
        : e.key === "ArrowLeft" || e.key === "ArrowUp"
          ? -1
          : 0;
    if (!step) return;
    e.preventDefault();
    const next = ids[(i + step + ids.length) % ids.length];
    onChange(next);
    refs.current[next]?.focus();
  };

  return (
    <div
      role="radiogroup"
      aria-label={label}
      className="flex items-center gap-2.5 py-1"
    >
      {ids.map((id) => {
        const selected = id === current;
        return (
          <ColorSwatch
            key={id}
            ref={(el) => {
              refs.current[id] = el;
            }}
            fill={fill(id)}
            selected={selected}
            ring={ring}
            role="radio"
            aria-checked={selected}
            aria-label={name(id)}
            title={name(id)}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(id)}
            onKeyDown={onKeyDown}
          />
        );
      })}
    </div>
  );
};
