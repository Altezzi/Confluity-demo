# Confluity Demo

This is a desktop app you can open and click through. It starts in a sample company, Apex Industrial, with example customers, employees, tickets, and records. Those records are there so the pages and profiles have something to show. They are not a live workspace.

The app keeps its own data, separate from any other Confluity install.

## What you can look through

- **Dashboard.** A home view of the workspace.
- **CRM.** Companies and contacts. Open Northwind Robotics and Maya Chen.
- **Support.** Tickets, including the fleet sync case, with comments and the customer on the record.
- **Projects.** Engineering issues linked from support, including a blocked sync bug.
- **Operations.** Products, assets, and an open RMA for unit NW-4412.
- **Finance.** An order, an invoice, a vendor, and a purchase order.
- **Portal.** A customer-facing view of public updates and knowledge.
- **Docs.** Internal and public articles tied to the same case.
- **Insights.** Tables over the workspace data.
- **Teams.** Department chats inside the app.
- **Search.** Find a person, ticket, or serial such as NW-4412.
- **Profiles.** Open an employee to see their role, position, and assigned work.
- **Assistant.** A side panel that can draft from the record you have open. Nothing is saved until it is approved.

## How a company shapes it

### Roles and positions

Employees are not stuck with a fixed job title. An admin builds roles the way a Discord server builds roles: each role is a pack of permissions, and a person can hold more than one.

Positions sit on an org chart under categories. A position can carry roles, so a Support Lead and a Support Agent can share a department and still see different things. Access follows the position and the roles on it, down to individual fields such as contact email or internal comments.

Open **Admin** and look at **Roles** and **Org chart**.

### Ticket forms

The support form is not one fixed layout. An admin adds fields, chooses the type (short text, multi-line, dropdown, checkbox, or number), sets the order, and decides which form is the default. A company can make the new-ticket screen match the way they already take a case.

Open **Admin → Ticket forms**.

### The side panel

Every section on the left is a module. An admin turns modules on or off for the workspace. If the company already uses Slack, Teams, or something else for chat, they turn **Teams** off and that item leaves the side panel. The same switch covers CRM, Support, Projects, Operations, Finance, Portal, Docs, Insights, and Search. People only see the modules their role allows and the workspace has switched on.

Open **Admin → Modules**.

## Where the sample lives

The window opens already inside the sample workspace. If you sign out, the next screen has **Open the sample workspace**.

Sample files stay in the operating system’s app data folder, under **Confluity Demo**.
