import type { Ticket, TicketForm, TicketFormField, TicketPriority, TicketStatus } from "@confluity/shared";

function systemField(key: string, label: string, required: boolean, position: number): TicketFormField {
  return {
    field_id: key,
    position,
    required,
    field: {
      id: key,
      key,
      label,
      kind: "system",
      options: [],
      hint: null,
      active: true,
      system: true,
    },
  };
}

/** Used only when the workspace has not loaded a saved form yet. */
export const fallbackTicketForm: TicketForm = {
  id: "",
  name: "General",
  description: null,
  active: true,
  is_default: true,
  position: 0,
  fields: [
    systemField("title", "Subject", true, 0),
    systemField("company", "Company", true, 1),
    systemField("contact", "Contact", false, 2),
    systemField("priority", "Priority", false, 3),
    systemField("status", "Status", false, 4),
    systemField("tags", "Tags", false, 5),
    systemField("description", "Description", false, 6),
  ],
};

export type TicketDraft = {
  title: string;
  description: string;
  companyId: string;
  contactId: string;
  priority: TicketPriority;
  status: TicketStatus;
  tags: string;
  assigneeId: string;
  custom: Record<string, string>;
};

export function emptyTicketDraft(companyId = ""): TicketDraft {
  return {
    title: "",
    description: "",
    companyId,
    contactId: "",
    priority: "medium",
    status: "open",
    tags: "",
    assigneeId: "",
    custom: {},
  };
}

export function draftFromTicket(ticket: Ticket): TicketDraft {
  return {
    title: ticket.title,
    description: ticket.description ?? "",
    companyId: ticket.company_id,
    contactId: ticket.contact_id ?? "",
    priority: ticket.priority,
    status: ticket.status,
    tags: ticket.tags ?? "",
    assigneeId: ticket.assignee_id ?? "",
    custom: { ...(ticket.custom_fields ?? {}) },
  };
}

export function missingRequired(form: TicketForm | null, draft: TicketDraft): string | null {
  if (!form) {
    if (!draft.title.trim()) return "Subject is required.";
    if (!draft.companyId) return "Company is required.";
    return null;
  }
  for (const placement of form.fields) {
    if (!placement.required) continue;
    const missing = (() => {
      switch (placement.field.key) {
        case "title":
          return !draft.title.trim();
        case "company":
          return !draft.companyId;
        case "contact":
          return !draft.contactId;
        case "description":
          return !draft.description.trim();
        case "tags":
          return !draft.tags.trim();
        case "assignee":
          return !draft.assigneeId;
        case "status":
        case "priority":
          return false;
        default: {
          const value = draft.custom[placement.field.id] ?? "";
          if (placement.field.kind === "checkbox") return value !== "true";
          return !value.trim();
        }
      }
    })();
    if (missing) return `${placement.field.label} is required.`;
  }
  return null;
}
