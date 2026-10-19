import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { Asset, Product, Rma } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function OperationsPage() {
  const { setFocus } = useApp();
  const [products, setProducts] = useState<Product[]>([]);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [rmas, setRmas] = useState<Rma[]>([]);

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Operations" });
    void (async () => {
      const [p, a, r] = await Promise.all([api.listProducts(), api.listAssets(), api.listRmas()]);
      setProducts(p);
      setAssets(a);
      setRmas(r);
    })();
  }, [setFocus]);

  const openRmas = rmas.filter((r) => r.status === "open" || r.status === "approved");

  return (
    <div className="page">
      <div className="home-hero">
        <div className="home-hero-copy">
          <h1>Operations</h1>
          <p>Products, serialized assets, warranty, and RMA.</p>
        </div>
        <div className="home-hero-actions">
          <Link className="btn btn-primary" to="/operations/products">
            Products
          </Link>
          <Link className="btn" to="/operations/assets">
            Assets
          </Link>
          <Link className="btn" to="/operations/rmas">
            RMAs
          </Link>
        </div>
      </div>

      {openRmas.length > 0 && (
        <Link className="alert-banner" to="/operations/rmas">
          {openRmas.length} open RMA{openRmas.length === 1 ? "" : "s"} need attention
        </Link>
      )}

      <div className="stat-row">
        <Link className="stat" to="/operations/products">
          <div className="value">{products.length}</div>
          <div className="label">Products</div>
        </Link>
        <Link className="stat" to="/operations/assets">
          <div className="value">{assets.length}</div>
          <div className="label">Assets</div>
        </Link>
        <Link className="stat" to="/operations/rmas">
          <div className="value">{openRmas.length}</div>
          <div className="label">Open RMAs</div>
        </Link>
        <Link className="stat" to="/operations/rmas">
          <div className="value">{rmas.length}</div>
          <div className="label">All RMAs</div>
        </Link>
      </div>

      <div className="grid-2">
        <div className="panel">
          <div className="panel-head">
            <h2>Recent assets</h2>
            <Link className="hint" to="/operations/assets">
              View all
            </Link>
          </div>
          <div className="list">
            {assets.slice(0, 5).map((a) => (
              <Link key={a.id} className="list-item" to={recordPath("asset", a.id)}>
                <div>
                  <div className="title">{a.name}</div>
                  <div className="meta">{a.serial || "No serial"}</div>
                </div>
              </Link>
            ))}
            {assets.length === 0 && <div className="empty">No assets yet</div>}
          </div>
        </div>
        <div className="panel">
          <div className="panel-head">
            <h2>Open RMAs</h2>
            <Link className="hint" to="/operations/rmas">
              View all
            </Link>
          </div>
          <div className="list">
            {openRmas.slice(0, 5).map((r) => (
              <Link key={r.id} className="list-item" to={recordPath("rma", r.id)}>
                <div>
                  <div className="title">{r.reason || "RMA"}</div>
                  <div className="meta">
                    <span className={`badge ${r.status}`}>{formatLabel(r.status)}</span>
                  </div>
                </div>
              </Link>
            ))}
            {openRmas.length === 0 && <div className="empty">No open RMAs</div>}
          </div>
        </div>
      </div>
    </div>
  );
}
