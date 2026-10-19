import type { Contact, Member } from "@confluity/shared";

const AVATAR_HUES = [208, 262, 172, 28, 332, 145, 18, 230];

export function personInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
}

export function avatarTone(name: string): { background: string; color: string } {
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  const hue = AVATAR_HUES[hash % AVATAR_HUES.length];
  return {
    background: `linear-gradient(160deg, hsl(${hue} 46% 38%), hsl(${hue} 38% 22%))`,
    color: `hsl(${hue} 85% 90%)`,
  };
}

export function departmentFromRole(role?: string | null): string {
  switch (role) {
    case "engineering":
      return "Engineering";
    case "support":
      return "Support";
    case "admin":
      return "Operations";
    case "viewer":
      return "Viewer";
    default:
      return role ?? "";
  }
}

export function memberDepartment(member?: Member | null): string {
  if (!member) return "";
  return member.title?.trim() || "";
}

/** Prefer explicit department/category label when provided. */
export function memberPlacement(
  member?: Member | null,
  categoryName?: string | null,
  positionName?: string | null,
): string {
  const pos = positionName?.trim() || member?.title?.trim() || "";
  const cat = categoryName?.trim() || "";
  if (cat && pos) return `${cat} · ${pos}`;
  return pos || cat || "";
}

export function resolvePerson(
  name: string,
  members: Member[],
  contacts: Contact[],
): { name: string; department: string } {
  const member = members.find((m) => m.name.toLowerCase() === name.toLowerCase());
  if (member) return { name: member.name, department: memberDepartment(member) };
  const contact = contacts.find((c) => c.name.toLowerCase() === name.toLowerCase());
  if (contact) return { name: contact.name, department: contact.title?.trim() || "Customer" };
  return { name, department: "" };
}

export function PersonChip({
  name,
  department,
  size = "md",
}: {
  name: string;
  department?: string | null;
  size?: "sm" | "md";
}) {
  const tone = avatarTone(name);
  return (
    <span className={`person-chip person-chip-${size}`}>
      <span className="person-avatar" style={tone} aria-hidden>
        {personInitials(name)}
      </span>
      <span className="person-name">{name}</span>
      {department ? <span className="person-dept">{department}</span> : null}
    </span>
  );
}
