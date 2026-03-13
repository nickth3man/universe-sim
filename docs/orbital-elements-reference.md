# Orbital Elements Reference for Universe-Sim

Authoritative orbital elements for solar system bodies used in universe-sim. All values at **J2000.0 epoch** (JD 2451545.0 = 2000-01-01 12:00 TDB) unless noted.

---

## 1. Planets (Heliocentric, J2000.0 Ecliptic)

**Source:** [JPL Approximate Positions of the Planets](https://ssd.jpl.nasa.gov/planets/approx_pos.html) — Keplerian elements and rates, mean ecliptic and equinox of J2000, valid 1800 AD–2050 AD. Based on Standish & Williams (1992), DE ephemeris.

**Note:** JPL provides elements for the **Earth–Moon Barycenter (EM Bary)**; Earth's heliocentric orbit is effectively the same. For "Earth" in the sim, use EM Bary elements.

| Planet | a (AU) | e | i (°) | i (rad) | Ω (°) | ω (°) | M at epoch (°) | P (days) |
|--------|--------|---|-------|---------|-------|-------|----------------|----------|
| Mercury | 0.38709927 | 0.20563593 | 7.00497902 | 0.122260 | 48.33076593 | 29.12703 | 174.79253 | 87.969 |
| Venus | 0.72333566 | 0.00677672 | 3.39467605 | 0.059248 | 76.67984255 | 54.92262 | 50.37663 | 224.701 |
| Earth (EM Bary) | 1.00000261 | 0.01671123 | −0.00001531 | −0.00000027 | 0.0 | 102.93768 | 357.52689 | 365.256 |
| Mars | 1.52371034 | 0.09339410 | 1.84969142 | 0.032283 | 49.55953891 | 286.49683 | 19.41248 | 686.980 |
| Jupiter | 5.20288700 | 0.04838624 | 1.30439695 | 0.022766 | 100.47390909 | 274.25452 | 19.66796 | 4332.82 |
| Saturn | 9.53667594 | 0.05386179 | 2.48599187 | 0.043384 | 113.66242448 | 338.93605 | 317.35537 | 10759.22 |
| Uranus | 19.18916464 | 0.04725744 | 0.77263783 | 0.013485 | 74.01692503 | 96.93735 | 142.28383 | 30687.15 |
| Neptune | 30.06992276 | 0.00859048 | 1.77004347 | 0.030889 | 131.78422574 | 273.17949 | 259.91521 | 60190.03 |

**Derived:** ω = ϖ − Ω, M = L − ϖ (from JPL L, ϖ, Ω).

**Orbital period** from Kepler's third law: P ≈ 365.256 × a^1.5 days (a in AU).

---

## 2. Earth's Moon (Geocentric)

**Source:** [JPL Planetary Satellite Mean Elements](https://ssd.jpl.nasa.gov/?sat_elem) — DE405/LE405, ecliptic frame, epoch 2000-01-01.5 TDB.

| Parameter | Value | Unit |
|-----------|-------|------|
| Semi-major axis | 384,400 | km |
| Semi-major axis | 0.0025696 | AU |
| Eccentricity | 0.0554 | — |
| Inclination | 5.16 | ° |
| Inclination | 0.09005 | rad |
| Longitude of ascending node | 125.08 | ° |
| Argument of periapsis | 318.15 | ° |
| Mean anomaly at epoch | 135.27 | ° |
| Orbital period | 27.322 | days |

**Note:** Mean elements are fit to the integrated orbit; for high-precision ephemerides use [JPL Horizons](https://ssd.jpl.nasa.gov/horizons/).

---

## 3. Jupiter's Galilean Moons (Jovicentric)

**Source:** [JPL Planetary Satellite Mean Elements](https://ssd.jpl.nasa.gov/?sat_elem) — JUP365 ephemeris, Laplace frame, epoch 2000-01-01.5 TDB.

| Moon | a (km) | e | i (°) | i (rad) | Ω (°) | ω (°) | M (°) | P (days) |
|------|--------|---|-------|---------|-------|-------|------|----------|
| Io | 421,800 | 0.004 | 0.0 | 0.0 | 0.0 | 49.1 | 330.9 | 1.769 |
| Europa | 671,100 | 0.009 | 0.5 | 0.00873 | 184.0 | 45.0 | 345.4 | 3.525 |
| Ganymede | 1,070,400 | 0.001 | 0.2 | 0.00349 | 58.5 | 198.3 | 324.8 | 7.156 |
| Callisto | 1,882,700 | 0.007 | 0.3 | 0.00524 | 309.1 | 43.8 | 87.4 | 16.690 |

**Note:** Galilean moons use the Laplace plane (Jupiter’s equator); inclinations and nodes are small. For Io, i and Ω are 0 in this frame.

---

## 4. Source URLs

| Source | URL |
|--------|-----|
| JPL Approximate Planetary Positions | https://ssd.jpl.nasa.gov/planets/approx_pos.html |
| JPL Planetary Satellite Mean Elements | https://ssd.jpl.nasa.gov/?sat_elem |
| JPL Horizons System | https://ssd.jpl.nasa.gov/horizons/ |
| JPL Orbits & Ephemerides | https://ssd.jpl.nasa.gov/planets/orbits.html |
| NASA Planetary Fact Sheets (NSSDC) | https://nssdc.gsfc.nasa.gov/planetary/factsheet/ |
| IAU Minor Planet Center (Orbital Elements) | https://www.minorplanetcenter.net/iau/info/OrbElsExplanation.html |

---

## 5. Conversion Notes

- **Degrees → radians:** multiply by π/180 ≈ 0.0174533
- **AU → km:** 1 AU = 149,597,870.7 km
- **Epoch:** J2000.0 = JD 2451545.0 = 2000-01-01 12:00:00 TDB
- **Mean anomaly at epoch:** M₀ = L − ϖ (mean longitude minus longitude of perihelion)
- **Argument of periapsis:** ω = ϖ − Ω

---

*Generated for universe-sim. Data from NASA JPL and NSSDC.*
