import React from "react";

/** Hairline so very pale swatches keep an edge on a light card. */
const HAIRLINE =
  "inset 0 0 0 1px color-mix(in srgb, var(--color-text) 10%, transparent)";
/** White ring touching the circle; App.css adds a faint outer line in light theme. */
const RING = "var(--swatch-ring)";
/** Black ring touching the circle, for the pale clear-glass colours. */
const RING_DARK = "0 0 0 2px #0b0b0c";
const GLIDE = "cubic-bezier(0.22, 1, 0.36, 1)";

interface ColorSwatchProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "color"> {
  /** CSS background (flat colour, or the hard-edged wheel). */
  fill: string;
  selected: boolean;
  /** "dark" puts a black ring on the chosen circle (easier to see on pale colours). */
  ring?: "light" | "dark";
}

/**
 * The one colour circle used by every colour row: flat 28px, a 2px white
 * ring with no gap when chosen, nothing else.
 */
export const ColorSwatch = React.forwardRef<
  HTMLButtonElement,
  ColorSwatchProps
>(({ fill, selected, ring = "light", style, className = "", ...rest }, ref) => (
  <button
    ref={ref}
    type="button"
    {...rest}
    className={`relative h-7 w-7 shrink-0 cursor-pointer rounded-full outline-none hover:scale-[1.06] active:scale-[0.97] focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-logo-primary/60 motion-reduce:transition-none! motion-reduce:hover:scale-100 motion-reduce:active:scale-100 ${className}`}
    style={{
      background: fill,
      boxShadow: selected ? (ring === "dark" ? RING_DARK : RING) : HAIRLINE,
      transition: `box-shadow 160ms ease-out, scale 200ms ${GLIDE}`,
      ...style,
    }}
  />
));
ColorSwatch.displayName = "ColorSwatch";

/** Swatch preview colours for the clear glass row (not what the bar renders). */
export const CLEAR_SWATCH_COLORS: Record<string, string> = {
  clear: "#d9d9dc",
  frost: "#f4f4f2",
  sand: "#e6d5b8",
  rose: "#e8c6c8",
  sage: "#c8dbc4",
  sky: "#c6d6ee",
};

/** Swatch preview colours for the black glass row (not what the bar renders). */
export const BLACK_SWATCH_COLORS: Record<string, string> = {
  black: "#18181a",
  graphite: "#3a3a40",
  navy: "#26305a",
  plum: "#4a2a4e",
  forest: "#26402c",
  cocoa: "#4a3324",
};
