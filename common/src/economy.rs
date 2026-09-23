//! Sistema de economía clásica inspirada en World of Warcraft (Cobre, Plata, Oro).
//! 100 Monedas de Cobre = 1 Moneda de Plata
//! 100 Monedas de Plata = 1 Moneda de Oro (10,000 Monedas de Cobre)

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    /// Total de valor expresado en monedas de cobre
    pub copper_total: u64,
}

impl Money {
    pub const COPPER_PER_SILVER: u64 = 100;
    pub const SILVER_PER_GOLD: u64 = 100;
    pub const COPPER_PER_GOLD: u64 = 100 * 100; // 10,000

    pub fn from_copper(copper: u64) -> Self {
        Self { copper_total: copper }
    }

    pub fn from_parts(gold: u64, silver: u64, copper: u64) -> Self {
        Self {
            copper_total: gold * Self::COPPER_PER_GOLD + silver * Self::COPPER_PER_SILVER + copper,
        }
    }

    #[inline]
    pub fn gold(&self) -> u64 {
        self.copper_total / Self::COPPER_PER_GOLD
    }

    #[inline]
    pub fn silver(&self) -> u64 {
        (self.copper_total % Self::COPPER_PER_GOLD) / Self::COPPER_PER_SILVER
    }

    #[inline]
    pub fn copper(&self) -> u64 {
        self.copper_total % Self::COPPER_PER_SILVER
    }

    /// Formatea el dinero al estilo WoW: "Xg Ys Zc"
    pub fn format_short(&self) -> String {
        let g = self.gold();
        let s = self.silver();
        let c = self.copper();

        if g > 0 {
            format!("{}g {}s {}c", g, s, c)
        } else if s > 0 {
            format!("{}s {}c", s, c)
        } else {
            format!("{}c", c)
        }
    }

    /// Formatea el dinero en español: "X de oro, Y de plata, Z de cobre"
    pub fn format_es(&self) -> String {
        let g = self.gold();
        let s = self.silver();
        let c = self.copper();

        let mut parts = Vec::new();
        if g > 0 {
            parts.push(format!("{} oro", g));
        }
        if s > 0 {
            parts.push(format!("{} plata", s));
        }
        if c > 0 || parts.is_empty() {
            parts.push(format!("{} cobre", c));
        }
        parts.join(", ")
    }
}
