import type { ComponentType, SVGProps } from "react";
import { cn } from "@/lib/utils";
import {
  clientIdFromDriverKind,
  getClientLabel,
  isSupportedClientId,
  type SupportedClientId,
} from "@/lib/clients";

type LogoProps = SVGProps<SVGSVGElement>;

function ClaudeLogo({ className, ...props }: LogoProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} {...props}>
      <rect width="24" height="24" rx="6" fill="#D97757" />
      <path
        d="M12 5.5c-2.2 0-4 1.5-4.5 3.6-.6-.3-1.3-.4-2-.2-1.5.4-2.4 1.9-2 3.4.3 1.1 1.2 1.9 2.3 2.1-.1.3-.1.6-.1.9 0 2.5 2 4.5 4.5 4.5s4.5-2 4.5-4.5c0-.3 0-.6-.1-.9 1.1-.2 2-1 2.3-2.1.4-1.5-.5-3-2-3.4-.7-.2-1.4-.1-2 .2C16 7 14.2 5.5 12 5.5Z"
        fill="#F4E8DF"
      />
    </svg>
  );
}

function CodexLogo({ className, ...props }: LogoProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} {...props}>
      <rect width="24" height="24" rx="6" fill="#10A37F" />
      <path
        d="M7 8.5h10M7 12h7M7 15.5h10"
        stroke="white"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
      <path
        d="M16.5 14.5 18 16l-1.5 1.5"
        stroke="white"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function OpenCodeLogo({ className, ...props }: LogoProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} {...props}>
      <rect width="24" height="24" rx="6" fill="#111827" className="dark:fill-[#E5E7EB]" />
      <path
        d="M8.5 8 6 12l2.5 4M15.5 8 18 12l-2.5 4M13 8l-2 8"
        stroke="white"
        className="dark:stroke-[#111827]"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function CursorLogo({ className, ...props }: LogoProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} {...props}>
      <rect width="24" height="24" rx="6" fill="#0F0F0F" className="dark:fill-[#F5F5F5]" />
      <path
        d="M7 7v10l3.2-2.8L12.5 18l1.8-1-2.1-3.4L17 11.5 7 7Z"
        fill="white"
        className="dark:fill-[#0F0F0F]"
      />
    </svg>
  );
}

function GeminiLogo({ className, ...props }: LogoProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} {...props}>
      <defs>
        <linearGradient id="gemini-gradient" x1="4" y1="4" x2="20" y2="20">
          <stop stopColor="#4285F4" />
          <stop offset="0.5" stopColor="#9B72F2" />
          <stop offset="1" stopColor="#D96570" />
        </linearGradient>
      </defs>
      <rect width="24" height="24" rx="6" fill="url(#gemini-gradient)" />
      <path
        d="M12 6.5c0 3-2.5 3-2.5 5.5S12 15.5 12 18.5c0-3 2.5-3 2.5-5.5S12 9.5 12 6.5Z"
        fill="white"
      />
      <path
        d="M6.5 12c3 0 3-2.5 5.5-2.5S15.5 12 18.5 12c-3 0-3 2.5-5.5 2.5S6.5 12 6.5 12Z"
        fill="white"
      />
    </svg>
  );
}

const CLIENT_LOGOS: Record<SupportedClientId, ComponentType<LogoProps>> = {
  "claude-code": ClaudeLogo,
  codex: CodexLogo,
  opencode: OpenCodeLogo,
  cursor: CursorLogo,
  "gemini-cli": GeminiLogo,
};

interface ClientLogoProps {
  clientId: string;
  driverKind?: string;
  className?: string;
  title?: string;
}

export function ClientLogo({
  clientId,
  driverKind,
  className,
  title,
}: ClientLogoProps) {
  const resolvedId = isSupportedClientId(clientId)
    ? clientId
    : driverKind
      ? clientIdFromDriverKind(driverKind)
      : null;

  if (!resolvedId) {
    return (
      <span
        className={cn(
          "inline-flex size-6 shrink-0 items-center justify-center rounded-md bg-muted text-[10px] font-semibold text-muted-foreground",
          className,
        )}
        title={title ?? clientId}
      >
        {clientId.slice(0, 1).toUpperCase()}
      </span>
    );
  }

  const Logo = CLIENT_LOGOS[resolvedId];
  const label = title ?? getClientLabel(resolvedId);

  return (
    <Logo
      className={cn("size-6 shrink-0 rounded-md", className)}
      role="img"
      aria-label={label}
    />
  );
}

interface ClientNameProps {
  clientId: string;
  name?: string;
  driverKind?: string;
  className?: string;
  logoClassName?: string;
}

export function ClientName({
  clientId,
  name,
  driverKind,
  className,
  logoClassName,
}: ClientNameProps) {
  const label = name ?? getClientLabel(clientId);

  return (
    <span className={cn("inline-flex min-w-0 items-center gap-2", className)}>
      <ClientLogo
        clientId={clientId}
        driverKind={driverKind}
        className={logoClassName}
        title={label}
      />
      <span className="truncate">{label}</span>
    </span>
  );
}
