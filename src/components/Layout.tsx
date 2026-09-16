import { Link, useRouterState } from "@tanstack/react-router";
import type { ReactNode } from "react";
import styles from "./Layout.module.css";

function HorseshoeMark() {
  return (
    <svg
      className={styles["nav-brand-mark"]}
      viewBox="0 0 32 32"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M16 4c-6 0-10 5-10 11 0 5 2.5 8.5 3.5 11a1 1 0 0 0 1.9-.6c-.6-2.1-1.4-4-1.4-7.4 0-5.6 3-8 6-8s6 2.4 6 8c0 3.4-.8 5.3-1.4 7.4a1 1 0 0 0 1.9.6c1-2.5 3.5-6 3.5-11 0-6-4-11-10-11Z"
        fill="currentColor"
      />
    </svg>
  );
}

function NavLink({ to, children }: { to: string; children: ReactNode }) {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const isActive =
    to === "/" ? pathname === "/" : pathname.startsWith(to);
  return (
    <Link
      to={to}
      className={`${styles["nav-link"]} ${isActive ? styles.active : ""}`}
    >
      {children}
    </Link>
  );
}

export function Layout({ children }: { children: ReactNode }) {
  return (
    <div className={styles.layout}>
      <div className={styles["nav-wrap"]}>
        <header className={styles.nav}>
          <Link to="/" className={styles["nav-brand"]}>
            <HorseshoeMark />
            <span>Paddock & Rein</span>
          </Link>
          <nav className={styles["nav-links"]}>
            <NavLink to="/">Search</NavLink>
            <NavLink to="/owner">Owner dashboard</NavLink>
          </nav>
        </header>
      </div>
      <main className={styles.main}>{children}</main>
      <footer className={styles.footer}>
        Paddock & Rein — booking horses and carriages, one pasture at a
        time.
      </footer>
    </div>
  );
}
