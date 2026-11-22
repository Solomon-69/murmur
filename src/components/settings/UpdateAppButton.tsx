import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SettingContainer } from "../ui/SettingContainer";
import { Button } from "../ui/Button";

const LABEL = "Update App";
const LABEL_BUSY = "Updating...";

interface UpdateAppButtonProps {
  grouped?: boolean;
}

/** Checks the latest GitHub release, downloads it, installs it and restarts. */
export const UpdateAppButton: React.FC<UpdateAppButtonProps> = ({
  grouped = false,
}) => {
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  const update = async () => {
    setBusy(true);
    setStatus("Checking for updates...");
    try {
      const result = await invoke<string>("update_app");
      setStatus(
        result === "latest" ? "You have the latest version" : String(result),
      );
    } catch (e) {
      setStatus(`Update failed: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  };

  return (
    <SettingContainer
      title="Update App"
      description={
        status ||
        "Download and install the newest murmur from GitHub, then restart."
      }
      descriptionMode="inline"
      grouped={grouped}
    >
      <Button variant="secondary" size="sm" disabled={busy} onClick={update}>
        {busy ? LABEL_BUSY : LABEL}
      </Button>
    </SettingContainer>
  );
};
