import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

function IconBase(props: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    />
  );
}

export function AppsIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <rect x="4" y="4" width="6" height="6" rx="1.5" />
      <rect x="14" y="4" width="6" height="6" rx="1.5" />
      <rect x="4" y="14" width="6" height="6" rx="1.5" />
      <rect x="14" y="14" width="6" height="6" rx="1.5" />
    </IconBase>
  );
}

export function PlusIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 5v14" />
      <path d="M5 12h14" />
    </IconBase>
  );
}

export function SearchIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <circle cx="11" cy="11" r="6.5" />
      <path d="m16 16 3.5 3.5" />
    </IconBase>
  );
}

export function SettingsIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 3.75 13.5 6l2.63.43.37 2.63 2 1.69-1 2.46 1 2.46-2 1.69-.37 2.63L13.5 18 12 20.25 10.5 18l-2.63-.43-.37-2.63-2-1.69 1-2.46-1-2.46 2-1.69.37-2.63L10.5 6 12 3.75Z" />
      <circle cx="12" cy="12" r="3" />
    </IconBase>
  );
}

export function PlayIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m8 6 10 6-10 6Z" />
    </IconBase>
  );
}

export function PencilIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m4 20 4.5-1 9.75-9.75a2.12 2.12 0 0 0-3-3L5.5 16 4 20Z" />
      <path d="m13.5 7.5 3 3" />
    </IconBase>
  );
}

export function TrashIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M4.5 7.5h15" />
      <path d="M9.5 7.5V5.75A1.75 1.75 0 0 1 11.25 4h1.5a1.75 1.75 0 0 1 1.75 1.75V7.5" />
      <path d="m6.5 7.5 1 11A1.75 1.75 0 0 0 9.24 20h5.52a1.75 1.75 0 0 0 1.74-1.5l1-11" />
      <path d="M10 11v5" />
      <path d="M14 11v5" />
    </IconBase>
  );
}

export function ExternalIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M14 5h5v5" />
      <path d="M10 14 19 5" />
      <path d="M19 13v4a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h4" />
    </IconBase>
  );
}

export function GlobeIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <circle cx="12" cy="12" r="8" />
      <path d="M4 12h16" />
      <path d="M12 4c2.5 2.3 4 5.2 4 8s-1.5 5.7-4 8c-2.5-2.3-4-5.2-4-8s1.5-5.7 4-8Z" />
    </IconBase>
  );
}

export function ShieldIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M12 3.5 6.5 5.75v5.5c0 4 2.4 6.78 5.5 9.25 3.1-2.47 5.5-5.25 5.5-9.25v-5.5L12 3.5Z" />
    </IconBase>
  );
}

export function BrowserIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <rect x="3.5" y="5" width="17" height="14" rx="3" />
      <path d="M3.5 9h17" />
      <path d="M8 7h.01" />
      <path d="M11 7h.01" />
    </IconBase>
  );
}

export function LayoutIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <rect x="4" y="5" width="16" height="14" rx="2.5" />
      <path d="M10 5v14" />
    </IconBase>
  );
}

export function TrayIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M4 7h16l-2 10H6L4 7Z" />
      <path d="M9 12a3 3 0 0 0 6 0" />
    </IconBase>
  );
}

export function SparkIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m12 3 1.45 4.55L18 9l-4.55 1.45L12 15l-1.45-4.55L6 9l4.55-1.45L12 3Z" />
      <path d="m18.5 15 .72 2.28 2.28.72-2.28.72-.72 2.28-.72-2.28-2.28-.72 2.28-.72.72-2.28Z" />
    </IconBase>
  );
}

export function CloseIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m6 6 12 12" />
      <path d="M18 6 6 18" />
    </IconBase>
  );
}

export function ArrowLeftIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m14.5 6.5-5 5 5 5" />
    </IconBase>
  );
}

export function ArrowRightIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m9.5 6.5 5 5-5 5" />
    </IconBase>
  );
}

export function HomeIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="m4 11 8-6 8 6" />
      <path d="M6.5 10.5V19h11v-8.5" />
    </IconBase>
  );
}

export function RefreshIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M19 12a7 7 0 1 1-2.05-4.95" />
      <path d="M19 5v5h-5" />
    </IconBase>
  );
}

export function DatabaseIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <ellipse cx="12" cy="6.5" rx="6.5" ry="2.5" />
      <path d="M5.5 6.5v5c0 1.38 2.91 2.5 6.5 2.5s6.5-1.12 6.5-2.5v-5" />
      <path d="M5.5 11.5v5c0 1.38 2.91 2.5 6.5 2.5s6.5-1.12 6.5-2.5v-5" />
    </IconBase>
  );
}

export function FolderIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M4 7.5h5l2 2H20v7.5A2 2 0 0 1 18 19H6a2 2 0 0 1-2-2V7.5Z" />
    </IconBase>
  );
}

export function FilterIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M4 6h16" />
      <path d="M7 12h10" />
      <path d="M10 18h4" />
    </IconBase>
  );
}

export function MenuIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M5 7h14" />
      <path d="M5 12h14" />
      <path d="M5 17h14" />
    </IconBase>
  );
}

export function MinimizeIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <path d="M6 12.5h12" />
    </IconBase>
  );
}

export function SquareIcon(props: IconProps) {
  return (
    <IconBase {...props}>
      <rect x="6" y="6" width="12" height="12" rx="1.5" />
    </IconBase>
  );
}
