pub mod solver;
use crate::PlayingCard;

/// A [`Choice`] is an option in a decision
///
/// See the types that implement this trait to get a better understanding
pub trait Choice: std::fmt::Debug {
    /// A score for this choice based on the card history
    ///
    /// The score represents the value of this choice with the given
    /// history, with 1.0 being the identity. For example, if the correct
    /// choice was made, then the output could be 2.0 (i.e. money is doubled),
    /// otherwise 0.0
    ///
    /// # Card History Note
    /// The card history is given backwards, with idx:`0` being the unseen
    /// card, idx:`1` being the last seen card, etc.
    fn score(&self, history: &[PlayingCard]) -> f64;

    /// The next decision to consider after this choice
    fn next_decision(&self) -> DiscreteDecision;
}
/// A [`DiscreteDecision`] is a list of all possible [`Choice`]s available
/// as an option in a decision
///
/// Usually, it's an enumeration of a type that implements [`Choice`]
pub struct DiscreteDecision {
    choices: Vec<Box<dyn Choice>>,
}
impl DiscreteDecision {
    /// Creates a new [`DiscreteDecision`] from the given choices
    pub fn new<T: Choice + 'static>(choices: impl IntoIterator<Item = T>) -> Self {
        DiscreteDecision {
            choices: choices
                .into_iter()
                .map(|c| Box::new(c) as Box<dyn Choice>)
                .collect(),
        }
    }
    /// Creates a new [`DiscreteDecision`] from the given choices, along with
    /// the option for a [`Cashout`]
    ///
    /// See [`Cashout`] documentation for more info on this choice
    pub fn new_with_cashout<T: Choice + 'static>(choices: impl IntoIterator<Item = T>) -> Self {
        let mut dd = Self::new(choices);
        dd.choices.push(Box::new(Cashout));
        dd
    }
    /// Creates a [`DiscreteDecision`] with no choices
    pub fn empty() -> Self {
        Self {
            choices: Vec::new(),
        }
    }
}
impl IntoIterator for DiscreteDecision {
    type Item = Box<dyn Choice>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.choices.into_iter()
    }
}

#[derive(Debug)]
struct Cashout;
impl Choice for Cashout {
    fn score(&self, _: &[PlayingCard]) -> f64 {
        1.0 // cashout gives identity no matter what
    }
    fn next_decision(&self) -> DiscreteDecision {
        DiscreteDecision::empty() // after cashout, no other decisions to make
    }
}
