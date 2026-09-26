"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ConnectionStatus } from "@/components/ConnectionStatus";

const links = [
  { href: "/", label: "Search" },
  { href: "/documents", label: "Documents" },
];

export function Sidebar() {
  const pathname = usePathname();

  const nav = (
    <nav className="flex gap-1 md:flex-col">
      {links.map((link) => {
        const active = pathname === link.href;

        return (
          <Link
            key={link.href}
            href={link.href}
            className={`rounded px-2 py-1.5 text-sm ${
              active ? "bg-[#161616] text-[#f5f5f5]" : "text-[#8d8d8d] hover:text-[#f5f5f5]"
            }`}
          >
            {link.label}
          </Link>
        );
      })}
    </nav>
  );

  return (
    <>
      <header className="flex items-center justify-between gap-4 border-b border-[#242424] px-4 py-3 md:hidden">
        <div className="flex min-w-0 items-center gap-3">
          <Link href="/" className="shrink-0 text-sm tracking-[0.14em] text-[#f5f5f5]">
            RECALL
          </Link>
          {nav}
        </div>
        <ConnectionStatus />
      </header>
      <aside className="sticky top-0 hidden h-screen w-56 shrink-0 flex-col border-r border-[#242424] px-4 py-6 md:flex">
        <Link href="/" className="px-2 text-sm tracking-[0.14em] text-[#f5f5f5]">
          RECALL
        </Link>
        <div className="mt-8">{nav}</div>
        <div className="mt-auto px-2">
          <ConnectionStatus />
        </div>
      </aside>
    </>
  );
}
