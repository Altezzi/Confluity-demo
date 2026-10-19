import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from "react";
import { Link, useNavigate } from "react-router-dom";
import type { Company, Invoice, Order, Product, PurchaseOrder, Vendor } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

function money(cents: number) {
  return `$${(cents / 100).toFixed(2)}`;
}

function ListShell({
  eyebrow,
  title,
  subtitle,
  backTo,
  newLabel,
  searchPlaceholder,
  query,
  onQuery,
  onNew,
  children,
  modalOpen,
  modalTitle,
  onClose,
  form,
}: {
  eyebrow: string;
  title: string;
  subtitle: string;
  backTo: string;
  newLabel: string;
  searchPlaceholder: string;
  query: string;
  onQuery: (v: string) => void;
  onNew: () => void;
  children: ReactNode;
  modalOpen: boolean;
  modalTitle: string;
  onClose: () => void;
  form: ReactNode;
}) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">{eyebrow}</div>
          <h1>{title}</h1>
          <p>{subtitle}</p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="button" onClick={onNew}>
            {newLabel}
          </button>
          <Link className="btn" to={backTo}>
            Back
          </Link>
        </div>
      </div>

      <div className="panel">
        <div className="docs-toolbar">
          <input
            className="docs-search"
            value={query}
            onChange={(e) => onQuery(e.target.value)}
            placeholder={searchPlaceholder}
          />
        </div>
        <div className="list">{children}</div>
      </div>

      {modalOpen && (
        <div className="modal-backdrop" onClick={onClose}>
          <div
            className="modal modal-form"
            role="dialog"
            aria-modal="true"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2>{modalTitle}</h2>
              <button className="btn btn-ghost" type="button" onClick={onClose}>
                Close
              </button>
            </div>
            {form}
          </div>
        </div>
      )}
    </div>
  );
}

export function InvoicesPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [orders, setOrders] = useState<Order[]>([]);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [companyId, setCompanyId] = useState("");
  const [orderId, setOrderId] = useState("");
  const [reference, setReference] = useState("");

  async function reload() {
    const [inv, o, c] = await Promise.all([
      api.listInvoices(),
      api.listOrders(),
      api.listCompanies(),
    ]);
    setInvoices(inv);
    setOrders(o);
    setCompanies(c);
    if (!companyId && c[0]) setCompanyId(c[0].id);
    if (!orderId && o[0]) setOrderId(o[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Leads" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return invoices;
    return invoices.filter((inv) =>
      `${inv.reference ?? ""} ${inv.status} ${inv.notes ?? ""}`.toLowerCase().includes(q),
    );
  }, [invoices, query]);

  function closeModal() {
    setOpen(false);
    setReference("");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    try {
      const inv = await api.createInvoice({
        company_id: companyId || null,
        order_id: orderId || null,
        reference: reference || null,
        amount_cents: 10000,
        lines: [{ description: reference || "Line", quantity: 1, unit_cents: 10000 }],
      });
      closeModal();
      await reload();
      navigate(recordPath("invoice", inv.id));
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Finance"
      title="Leads"
      subtitle="Pipeline opportunities and billing status."
      backTo="/finance"
      newLabel="New lead"
      searchPlaceholder="Search leads…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="New lead"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Company</label>
            <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
              {companies.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Order</label>
            <select value={orderId} onChange={(e) => setOrderId(e.target.value)}>
              <option value="">None</option>
              {orders.map((o) => (
                <option key={o.id} value={o.id}>
                  {o.reference || o.id.slice(0, 8)}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Reference</label>
            <input
              autoFocus
              value={reference}
              onChange={(e) => setReference(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy}>
              Create lead
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((inv) => (
        <Link key={inv.id} className="list-item" to={recordPath("invoice", inv.id)}>
          <div>
            <div className="title">{inv.reference || "Lead"}</div>
            <div className="meta">
              <span className={`badge ${inv.status}`}>{formatLabel(inv.status)}</span>{" "}
              {money(inv.amount_cents)}
            </div>
          </div>
        </Link>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching leads" : "No leads yet"}</div>
      )}
    </ListShell>
  );
}

export function OrdersPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [orders, setOrders] = useState<Order[]>([]);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [companyId, setCompanyId] = useState("");
  const [reference, setReference] = useState("");

  async function reload() {
    const [o, c] = await Promise.all([api.listOrders(), api.listCompanies()]);
    setOrders(o);
    setCompanies(c);
    if (!companyId && c[0]) setCompanyId(c[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Orders" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return orders;
    return orders.filter((o) =>
      `${o.reference ?? ""} ${o.status} ${o.notes ?? ""}`.toLowerCase().includes(q),
    );
  }, [orders, query]);

  function closeModal() {
    setOpen(false);
    setReference("");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    try {
      const order = await api.createOrder({
        company_id: companyId || null,
        reference: reference || null,
        total_cents: 10000,
      });
      closeModal();
      await reload();
      navigate(recordPath("order", order.id));
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Finance"
      title="Orders"
      subtitle="Sales orders tied to customer accounts."
      backTo="/finance"
      newLabel="New order"
      searchPlaceholder="Search orders…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="New order"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Company</label>
            <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
              {companies.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Reference</label>
            <input
              autoFocus
              value={reference}
              onChange={(e) => setReference(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy}>
              Create order
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((o) => (
        <Link key={o.id} className="list-item" to={recordPath("order", o.id)}>
          <div>
            <div className="title">{o.reference || "Order"}</div>
            <div className="meta">
              <span className={`badge ${o.status}`}>{formatLabel(o.status)}</span>{" "}
              {money(o.total_cents)}
            </div>
          </div>
        </Link>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching orders" : "No orders yet"}</div>
      )}
    </ListShell>
  );
}

export function VendorsPage() {
  const { setFocus } = useApp();
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");

  async function reload() {
    setVendors(await api.listVendors());
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Vendors" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return vendors;
    return vendors.filter((v) =>
      `${v.name} ${v.email ?? ""} ${v.notes ?? ""}`.toLowerCase().includes(q),
    );
  }, [vendors, query]);

  function closeModal() {
    setOpen(false);
    setName("");
    setEmail("");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      await api.createVendor({ name: name.trim(), email: email || null });
      closeModal();
      await reload();
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Finance"
      title="Vendors"
      subtitle="Suppliers for purchase orders and restock."
      backTo="/finance"
      newLabel="New vendor"
      searchPlaceholder="Search vendors…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="New vendor"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Name</label>
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="form-row">
            <label>Email</label>
            <input value={email} onChange={(e) => setEmail(e.target.value)} />
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy || !name.trim()}>
              Create vendor
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((v) => (
        <div key={v.id} className="list-item">
          <div>
            <div className="title">{v.name}</div>
            <div className="meta">{v.email || "No email"}</div>
          </div>
        </div>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching vendors" : "No vendors yet"}</div>
      )}
    </ListShell>
  );
}

export function PurchaseOrdersPage() {
  const { setFocus } = useApp();
  const [pos, setPos] = useState<PurchaseOrder[]>([]);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [vendorId, setVendorId] = useState("");
  const [productId, setProductId] = useState("");
  const [reference, setReference] = useState("");
  const [qty, setQty] = useState("10");

  async function reload() {
    const [p, v, prod] = await Promise.all([
      api.listPurchaseOrders(),
      api.listVendors(),
      api.listProducts(),
    ]);
    setPos(p);
    setVendors(v);
    setProducts(prod);
    if (!vendorId && v[0]) setVendorId(v[0].id);
    if (!productId && prod[0]) setProductId(prod[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Purchase orders" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return pos;
    return pos.filter((p) =>
      `${p.reference ?? ""} ${p.status} ${p.notes ?? ""}`.toLowerCase().includes(q),
    );
  }, [pos, query]);

  function closeModal() {
    setOpen(false);
    setReference("");
    setQty("10");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    try {
      await api.createPurchaseOrder({
        vendor_id: vendorId || null,
        product_id: productId || null,
        reference: reference || null,
        quantity: Number(qty) || 1,
        unit_cents: 89000,
      });
      closeModal();
      await reload();
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Finance"
      title="Purchase orders"
      subtitle="Vendor restock and inventory replenishment."
      backTo="/finance"
      newLabel="New PO"
      searchPlaceholder="Search purchase orders…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="New purchase order"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Vendor</label>
            <select value={vendorId} onChange={(e) => setVendorId(e.target.value)}>
              {vendors.map((v) => (
                <option key={v.id} value={v.id}>
                  {v.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Product</label>
            <select value={productId} onChange={(e) => setProductId(e.target.value)}>
              {products.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Reference</label>
            <input
              autoFocus
              value={reference}
              onChange={(e) => setReference(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="form-row">
            <label>Quantity</label>
            <input value={qty} onChange={(e) => setQty(e.target.value)} />
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy}>
              Create PO
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((p) => (
        <div key={p.id} className="list-item">
          <div>
            <div className="title">{p.reference || "PO"}</div>
            <div className="meta">
              <span className={`badge ${p.status}`}>{formatLabel(p.status)}</span> qty {p.quantity}
            </div>
          </div>
        </div>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching purchase orders" : "No purchase orders yet"}</div>
      )}
    </ListShell>
  );
}
