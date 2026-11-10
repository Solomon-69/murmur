import React, { useLayoutEffect, useRef, useState } from "react";

export interface SegmentOption {
  value: string;
  label: string;
}

interface SegmentedControlProps {
  options: SegmentOption[];
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string;
}

/**
 * A segmented choice whose highlight slides once with a spring. The selection
 * flips the instant it is clicked (local state first, the save happens after),
 * so it never snaps back while the setting is being written. No focus ring.
 */
export const SegmentedControl: React.FC<SegmentedControlProps> = ({
  options,
  value,
  onChange,
  ariaLabel,
}) => {
  const [selected, setSelected] = useState(value);
  const [pill, setPill] = useState<{ left: number; width: number }>();
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const pending = useRef(false);

  // Follow outside changes, but never while our own click is being saved.
  useLayoutEffect(() => {
    if (!pending.current) setSelected(value);
  }, [value]);

  useLayoutEffect(() => {
    const el = refs.current[selected];
    if (el) setPill({ left: el.offsetLeft, width: el.offsetWidth });
  }, [selected, options]);

  const choose = async (next: string) => {
    if (next === selected) return;
    setSelected(next);
    pending.current = true;
    try {
      await Promise.resolve(onChange(next));
    } finally {
      pending.current = false;
    }
  };

  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className="relative inline-flex rounded-[10px] bg-wash p-[3px] select-none"
    >
      {pill && (
        <span
          aria-hidden="true"
          className="absolute top-[3px] bottom-[3px] rounded-[8px] bg-background shadow-sm"
          style={{
            left: pill.left,
            width: pill.width,
            transition:
              "left 420ms cubic-bezier(0.34, 1.36, 0.64, 1), width 420ms cubic-bezier(0.34, 1.36, 0.64, 1)",
          }}
        />
      )}
      {options.map((option) => (
        <button
          key={option.value}
          ref={(el) => {
            refs.current[option.value] = el;
          }}
          type="button"
          role="radio"
          aria-checked={selected === option.value}
          onClick={() => choose(option.value)}
          className={`relative z-10 min-h-[28px] px-3 text-center text-xs font-medium rounded-[8px] outline-none focus:outline-none focus-visible:outline-none transition-colors duration-200 ${
            selected === option.value ? "text-text" : "text-ink-2 hover:text-text"
          }`}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
};
