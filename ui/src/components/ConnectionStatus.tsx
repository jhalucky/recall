"use client";

import { useEffect, useState } from "react";
import { checkHealth } from "@/lib/api";

type Status = "checking" | "connected" | "disconnected";

export function ConnectionStatus() {
  const [status, setStatus] = useState<Status>("checking");

  useEffect(() => {
    let cancelled = false;

    checkHealth().then((ok) => {
      if (!cancelled) {
        setStatus(ok ? "connected" : "disconnected");
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  const label =
    status === "connected" ? "Connected" : status === "disconnected" ? "Disconnected" : "Checking";

  const color =
    status === "connected" ? "text-[#c6e08a]" : status === "disconnected" ? "text-[#d7a0a0]" : "text-[#6f6f6f]";

  return (
    <div className="text-xs leading-5">
      <div className="text-[#6f6f6f]">API</div>
      <div className={color}>{label}</div>
    </div>
  );
}
