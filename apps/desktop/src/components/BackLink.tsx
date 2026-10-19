import { Link } from "react-router-dom";

export function BackLink({ to, children }: { to: string; children: string }) {
  return (
    <div className="back-row">
      <Link className="back-link" to={to}>
        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden>
          <path
            d="M10 3.5 5.5 8 10 12.5"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.6"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
        {children}
      </Link>
    </div>
  );
}
