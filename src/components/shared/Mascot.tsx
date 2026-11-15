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
    className={`relative block select-none ${animation === "wave" ? "animate-rise" : animation === "slap" ? "animate-slap" : ""}`}
  >
    <img
      src={appIcon}
      alt=""
      draggable={false}
      className="block size-full drop-shadow-[0_6px_14px_rgba(0,0,0,0.18)]"
    />
  </span>
);
