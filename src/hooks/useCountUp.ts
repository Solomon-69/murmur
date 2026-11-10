import { useEffect, useRef, useState } from "react";

/** Length of the count-up, in milliseconds. */
const COUNT_UP_MS = 900;

const prefersReducedMotion = () => {
  try {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  } catch {
    return false;
  }
};

/**
 * Counts from the previous value up (or down) to `target` with an ease-out
 * curve. Jumps straight to the value when the person asked for reduced motion.
 */
export const useCountUp = (target: number, durationMs = COUNT_UP_MS) => {
  const [value, setValue] = useState(0);
  const latest = useRef(0);

  useEffect(() => {
    if (prefersReducedMotion() || durationMs <= 0) {
      latest.current = target;
      setValue(target);
      return;
    }
    let frame = 0;
    const from = latest.current;
    const start = performance.now();
    const step = (now: number) => {
      const progress = Math.min(1, (now - start) / durationMs);
      const eased = 1 - Math.pow(1 - progress, 3);
      latest.current = Math.round(from + (target - from) * eased);
      setValue(latest.current);
      if (progress < 1) frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [target, durationMs]);

  return value;
};
