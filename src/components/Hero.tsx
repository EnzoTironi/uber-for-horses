import type { ReactNode } from "react";
import styles from "./Hero.module.css";

export function Hero({ children }: { children?: ReactNode }) {
  return (
    <section className={styles.hero}>
      <div className={styles["hero-bg"]} aria-hidden="true" />
      <div className={styles["hero-content"]}>
        <p className={styles["hero-eyebrow"]}>Uber for horses</p>
        <h1 className={styles["hero-title"]}>
          Book a horse or a carriage,
          <br />
          straight from <em>the pasture</em>.
        </h1>
        <p className={styles["hero-subtitle"]}>
          Trail rides, wedding carriages, riding lessons — find a trusted
          horse or carriage near you, see real availability, and request your
          slot in minutes.
        </p>
        {children ? (
          <div className={styles["hero-card"]}>{children}</div>
        ) : null}
      </div>
      <p className={styles["hero-attribution"]}>
        Photo by{" "}
        <a
          href="https://unsplash.com/photos/12u0hawVCJ0"
          target="_blank"
          rel="noreferrer"
        >
          Jed Owen
        </a>{" "}
        on Unsplash
      </p>
    </section>
  );
}
