use super::{Choice, DiscreteDecision};
use crate::PlayingCard;

pub struct DiscreteDecisionTree {
    choices: Vec<EvalChoice>,
}
impl DiscreteDecisionTree {
    pub fn solve(decision: DiscreteDecision) -> Self {
        Self::compute(decision, 1.0, &[])
    }

    /// Computes the DDTree (evaluates all choices in the decision) for the
    /// given "pot" value and card history
    ///
    /// # First Call
    /// * `value` = bet size, or 1.0 if calculating expected values.
    /// * `history` = &[] (unless you wanna perform some schenanigans w/ small sets of cards)
    fn compute(decision: DiscreteDecision, value: f64, history: &[PlayingCard]) -> Self {
        let children = decision
            .into_iter()
            .map(|choice| EvalChoice::evaluate(choice, value, history))
            .collect();
        Self { choices: children }
    }

    pub fn optimal(&self) -> Option<&EvalChoice> {
        self.choices
            .iter()
            .max_by(|c1, c2| f64::total_cmp(&c1.expected_value, &c2.expected_value))
    }
    pub fn choices(&self) -> &[EvalChoice] {
        &self.choices
    }
}
pub struct EvalChoice {
    pub choice: Box<dyn Choice>,
    pub expected_value: f64,
    random_events: Vec<(PlayingCard, RandomEventOutcome)>,
}

impl EvalChoice {
    fn evaluate(choice: Box<dyn Choice>, value: f64, history: &[PlayingCard]) -> Self {
        // sum of all expected values, used to get average expected value
        // for this choice over all random events
        let mut ev_sum = 0.0;
        let mut all_random_events = Vec::with_capacity(52);

        // compute the EV for each random event given the choice,
        // then average all EVs (since each event is equally likely) to get
        // the overall EV for this choice
        let card_iter = PlayingCard::deck_iter().filter(|card| !history.contains(card));
        for card in card_iter {
            let random_event = RandomEventOutcome::evaluate(card, &*choice, value, history);
            ev_sum += random_event.expected_value();
            all_random_events.push((card, random_event));
        }

        let expected_value = ev_sum / all_random_events.len() as f64;
        Self {
            choice,
            expected_value,
            random_events: all_random_events,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(PlayingCard, RandomEventOutcome)> {
        self.random_events.iter()
    }
    pub fn get(&self, event: PlayingCard) -> Option<&RandomEventOutcome> {
        self.iter()
            .find(|(card, _)| *card == event)
            .map(|(_, outcome)| outcome)
    }
}

pub enum RandomEventOutcome {
    Lost,
    Leaf {
        value: f64,
    },
    Child {
        expected_value: f64,
        child: DiscreteDecisionTree,
    },
}
impl RandomEventOutcome {
    fn evaluate(
        event: PlayingCard,
        choice: &dyn Choice,
        value: f64,
        history: &[PlayingCard],
    ) -> Self {
        // create a new history with this card prepended (essentially a backwards history)
        let mut new_history = Vec::with_capacity(history.len() + 1);
        new_history.push(event);
        new_history.extend_from_slice(history);

        // calculate the outcome "value" for this choice+card
        let value = value * choice.score(&new_history);
        if value < 1e-6 {
            return RandomEventOutcome::Lost; // we lost since the outcome is basically zero
        }

        // compute the decision tree for the next decision
        let child = DiscreteDecisionTree::compute(choice.next_decision(), value, &new_history);
        if let Some(expected_value) = child.optimal().map(|ec| ec.expected_value) {
            // use the optimal choice's expected value for this outcome
            RandomEventOutcome::Child {
                expected_value,
                child,
            }
        } else {
            // since the next_decision's tree didn't have any choices,
            // then this is a leaf node and we should use its value
            RandomEventOutcome::Leaf { value }
        }
    }

    pub fn expected_value(&self) -> f64 {
        match self {
            RandomEventOutcome::Lost => 0.0,
            RandomEventOutcome::Leaf { value } => *value,
            RandomEventOutcome::Child {
                expected_value,
                child: _,
            } => *expected_value,
        }
    }
}
