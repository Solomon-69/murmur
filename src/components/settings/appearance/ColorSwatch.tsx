import React from "react";
import { Tooltip } from "@/components/ui/Tooltip";

/** Hairline so very pale swatches keep an edge on a light card. */
const HAIRLINE =
  "inset 0 0 0 1px color-mix(in srgb, var(--color-text) 18%, transparent)";
/** White ring touching the circle; App.css adds a faint outer line in light theme. */
const RING = "var(--swatch-ring)";
/** Two-tone ring touching the circle: dark inside, light outside, so the
 *  chosen colour shows on both the light and the dark app appearance. */
const RING_DARK = "0 0 0 2px #1c1c1e, 0 0 0 4px #f2f2f4";
const GLIDE = "cubic-bezier(0.22, 1, 0.36, 1)";

interface ColorSwatchProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "color"> {
  /** CSS background (flat colour, or the hard-edged wheel). */
  fill: string;
  selected: boolean;
  /** "dark" puts the two-tone ring on the chosen circle (glass colour rows). */
  ring?: "light" | "dark";
}

/**
 * The one colour circle used by every colour row: flat 28px, a 2px white
 * ring with no gap when chosen, nothing else.
 */
export const ColorSwatch = React.forwardRef<
  HTMLButtonElement,
  ColorSwatchProps
>(
  (
    { fill, selected, ring = "light", style, className = "", title, ...rest },
    ref,
  ) => {
    const button = (
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
    );
    // The colour's name shows in the app's own tooltip, not the system one.
    return title ? <Tooltip text={title}>{button}</Tooltip> : button;
  },
);
ColorSwatch.displayName = "ColorSwatch";

/** Swatch preview colours for the clear glass row (not what the bar renders). */
export const CLEAR_SWATCH_COLORS: Record<string, string> = {
  clear: "#c4c5ca",
  sand: "#d6ba8c",
  clay: "#c47a5c",
  sage: "#8aa078",
  ochre: "#cc983c",
  slate: "#708aa6",
};

/** Swatch preview colours for the black glass row (not what the bar renders).
 *  One solid colour each, lifted a little so the dark tints are easy to tell apart. */
export const BLACK_SWATCH_COLORS: Record<string, string> = {
  black: "#18181a",
  cocoa: "#5e3d27",
  rust: "#7c3520",
  moss: "#3e5a35",
  bronze: "#76591f",
  slate: "#34496a",
};
