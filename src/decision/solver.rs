use super::{Choice, DiscreteDecision};
use crate::PlayingCard;

pub struct DiscreteDecisionTree {
    choices: Vec<EvalChoice>,
    outcomes: usize,
}
impl DiscreteDecisionTree {
    /// Create/compute a decision tree from a starting decision
    pub fn solve(first_decision: DiscreteDecision) -> Self {
        Self::compute(first_decision, 1.0, &[])
    }

    /// Computes the DDTree (evaluates all choices in the decision) for the
    /// given "pot" value and card history
    ///
    /// # First Call
    /// * `value` = bet size, or 1.0 if calculating expected values
    /// * `history` = &[] (unless you wanna perform some schenanigans w/ small sets of cards)
    fn compute(decision: DiscreteDecision, value: f64, history: &[PlayingCard]) -> Self {
        // evaluate each choice recursively
        let evaluated_choices = decision
            .into_iter()
            .map(|choice| EvalChoice::evaluate(choice, value, history))
            .collect::<Vec<_>>();

        // find the # of outcomes by summing the count at each card+choice
        let outcomes = evaluated_choices
            .iter()
            .map(|c| c.iter().map(|(_, outcome)| outcome.count()).sum::<usize>())
            .sum();

        Self {
            choices: evaluated_choices,
            outcomes,
        }
    }

    /// Finds the optimal choice (or `None` if no choices are available)
    ///
    /// The optimal choice is the choice with the highest expected value
    pub fn optimal(&self) -> Option<&EvalChoice> {
        self.choices
            .iter()
            .max_by(|c1, c2| f64::total_cmp(&c1.expected_value, &c2.expected_value))
    }
    /// An iterator over all evaluated choices
    pub fn iter(&self) -> impl Iterator<Item = &EvalChoice> {
        self.choices.iter()
    }
    /// The total number of outcomes in this decision tree
    ///
    /// Equivellent to the # of leaf nodes in the tree
    pub fn outcome_count(&self) -> usize {
        self.outcomes
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
        let mut all_random_events = Vec::with_capacity(52); // TODO: unhardcode

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

    /// An iterator over all random events and their outcomes
    pub fn iter(&self) -> impl Iterator<Item = &(PlayingCard, RandomEventOutcome)> {
        self.random_events.iter()
    }
    /// Get an outcome based on the random event
    pub fn get(&self, event: PlayingCard) -> Option<&RandomEventOutcome> {
        self.iter()
            .find(|(card, _)| *card == event)
            .map(|(_, outcome)| outcome)
    }
}

pub enum RandomEventOutcome {
    Lost,
    Won {
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
            RandomEventOutcome::Won { value }
        }
    }

    pub fn expected_value(&self) -> f64 {
        match self {
            RandomEventOutcome::Lost => 0.0,
            RandomEventOutcome::Won { value } => *value,
            RandomEventOutcome::Child {
                expected_value,
                child: _,
            } => *expected_value,
        }
    }

    /// The count is the total number of outcomes for this event
    ///
    /// If there is no child, then it is trivially 1. Otherwise, it's the number
    /// of outcomes in the child
    pub fn count(&self) -> usize {
        match self {
            RandomEventOutcome::Lost => 1,
            RandomEventOutcome::Won { value: _ } => 1,
            RandomEventOutcome::Child {
                expected_value: _,
                child,
            } => child.outcomes,
        }
    }
}
