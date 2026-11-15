import React from "react";
import appIcon from "@/assets/app-icon.png";

interface MascotProps {
  /** Height in px. */
  size: number;
  /** Rise in once on arrival, or bounce (re-key it to play again). */
  animation?: "wave" | "slap" | null;
}

/** murmur's brand mark: the app icon (white sound wave on a dark tile). */
export const Mascot: React.FC<MascotProps> = ({ size, animation = null }) => (
  <span
    style={{ width: size, height: size }}
    className={`relative block overflow-hidden rounded-[30%] shadow-[0_4px_12px_rgba(0,0,0,0.16)] select-none ${animation === "wave" ? "animate-rise" : animation === "slap" ? "animate-slap" : ""}`}
  >
    <img
      src={appIcon}
      alt=""
      draggable={false}
      className="block size-full"
    />
  </span>
);
