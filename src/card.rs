use std::str::FromStr;

/// Simple representation of a standard playing card using values 0-52
///
/// First (least significant) 4 bits are used to determine suit
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlayingCard(u8);
impl PlayingCard {
    /// * hearts == 0
    /// * diamonds == 1
    /// * spades == 2
    /// * clubs == 3
    pub fn suit(self) -> u8 {
        self.0 & 0b11
    }
    /// * red == 0
    /// * black == 1
    pub fn color(self) -> u8 {
        (self.0 & 0b10) >> 1
    }
    /// 2-14
    /// * 11 = Jack
    /// * 12 = Queen
    /// * 13 = King
    /// * 14 = Ace
    pub fn rank(self) -> u8 {
        // remove suit bits
        (self.0 >> 2) + 2
    }

    /// An iterator over an entire deck of playing cards
    pub fn deck_iter() -> impl Iterator<Item = Self> {
        (0..52).map(Self)
    }
}

const RANK_LABELS: &[&str] = &[
    "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
];
const SUIT_LABELS: &[&str] = &["H", "D", "S", "C"];

impl std::fmt::Display for PlayingCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            RANK_LABELS[(self.rank() - 2) as usize],
            SUIT_LABELS[self.suit() as usize]
        )
    }
}

pub struct InvalidCardError;
impl FromStr for PlayingCard {
    type Err = InvalidCardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_uppercase();
        // kinda shit but it works and is fast enough
        Self::deck_iter()
            .find(|c| format!("{}", c) == s)
            .ok_or(InvalidCardError)
    }
}
