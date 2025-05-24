use super::{Choice, DiscreteDecision};
use crate::PlayingCard;

/// An Evaluated Decision
///
/// The decision tree evaluates each choice in a decision. Each [`ChoiceEval`]
/// then evaluates many [`RandomEventOutcome`]s, which ultimately may evaluate
/// another [`DiscreteDecisionTree`], hence the "tree" in the name
///
/// # Optimal Choice
/// In a decision, you have the ability to select which choice you want.
/// The [`DiscreteDecisionTree::optimal`] choice is the [`ChoiceEval`] with the highest
/// expected value (EV), i.e. it's the choice you want to select for the best returns
pub struct DiscreteDecisionTree {
    choices: Vec<ChoiceEval>,
    outcomes: usize,
}
impl DiscreteDecisionTree {
    /// Create/compute a decision tree from a starting decision
    pub fn solve(first_decision: DiscreteDecision) -> Self {
        Self::compute(first_decision, 1.0, &[])
    }

    /// Computes the DDTree (evaluates all choices in the decision) for the
    /// given pot value and card history
    ///
    /// # First Call
    /// * `pot` = bet size, or 1.0 if calculating expected values
    /// * `history` = &[] (unless you wanna perform some shenanigans w/ small sets of cards)
    fn compute(decision: DiscreteDecision, pot: f64, history: &[PlayingCard]) -> Self {
        // evaluate each choice recursively
        let evaluated_choices = decision
            .into_iter()
            .map(|choice| ChoiceEval::evaluate(choice, pot, history))
            .collect::<Vec<_>>();

        // find the # of outcomes by summing the count at each outcome
        let outcomes = evaluated_choices
            .iter()
            .flat_map(|c| c.iter())
            .map(|o| o.count())
            .sum();

        Self {
            choices: evaluated_choices,
            outcomes,
        }
    }

    /// Finds the optimal choice (or `None` if no choices are available)
    ///
    /// The optimal choice is the choice with the highest expected value
    pub fn optimal(&self) -> Option<&ChoiceEval> {
        self.choices
            .iter()
            .max_by(|c1, c2| f64::total_cmp(&c1.expected_value, &c2.expected_value))
    }
    /// An iterator over all evaluated choices
    pub fn iter(&self) -> impl Iterator<Item = &ChoiceEval> {
        self.choices.iter()
    }
    /// The total number of outcomes in this decision tree
    ///
    /// Equivalent to the # of leaf nodes in the tree
    pub fn outcome_count(&self) -> usize {
        self.outcomes
    }
}

/// An Evaluated Choice
///
/// A choice is evaluated by evaluating a [`RandomEventOutcome`] for every
/// random event possible with this choice, then averaging the [`RandomEventOutcome::value`]
/// of each  outcome (since each RE is equally likely)
pub struct ChoiceEval {
    pub choice: Box<dyn Choice>,
    pub expected_value: f64,
    random_events: Vec<RandomEventOutcome>,
}
impl ChoiceEval {
    /// Compute a [`ChoiceEval`] for the given choice, pot, and RE history
    fn evaluate(choice: Box<dyn Choice>, pot: f64, history: &[PlayingCard]) -> Self {
        // sum of all expected values, used to get average expected value
        // for this choice over all random events
        let mut ev_sum = 0.0;
        let mut all_random_events = Vec::with_capacity(52); // TODO: unhardcode

        // compute the EV for each random event given the choice,
        // then average all EVs (since each event is equally likely) to get
        // the overall EV for this choice
        let card_iter = PlayingCard::deck_iter().filter(|card| !history.contains(card));
        for card in card_iter {
            let random_event = RandomEventOutcome::evaluate(card, &*choice, pot, history);
            ev_sum += random_event.value;
            all_random_events.push(random_event);
        }

        let expected_value = ev_sum / all_random_events.len() as f64;
        Self {
            choice,
            expected_value,
            random_events: all_random_events,
        }
    }

    /// An iterator over all random events and their outcomes
    pub fn iter(&self) -> impl Iterator<Item = &RandomEventOutcome> {
        self.random_events.iter()
    }
    /// Get an outcome based on the random event
    pub fn get(&self, event: PlayingCard) -> Option<&RandomEventOutcome> {
        self.iter().find(|outcome| outcome.event == event)
    }
}


/// An Evaluated Random Event (RE) for a [`Choice`] (evaluated choice+card)
///
/// A random event is evaluated by finding the [`Choice::score`] of the choice+card:
/// 1. If `score` == 0.0, then we lost and the value is 0.0
/// 2. Elif `choice.next_decision().is_some()`, then we evaluate that decision with
/// [`DiscreteDecisionTree`] using `new_pot = pot * score` and use the optimal choice's EV
/// as the value
/// 3. Else, because there's no next decision, the value is the `pot * score`
///
/// # Score
/// The idea of a score from [`Choice::score`] might be confusing, but it's
/// essentially the pot multiplier depending on the RE. For example, if you
/// chose Red and got a red card, the score would be `2.0` (2x multiplier),
/// and if you got a black card the score would be `0.0` (you lost your money)
pub struct RandomEventOutcome {
    pub event: PlayingCard,
    pub value: f64,
    next_decision_tree: Option<DiscreteDecisionTree>,
}
impl RandomEventOutcome {
    /// Evaluate the outcome (most importantly value) of a choice+card
    /// (Random Event given a choice)
    fn evaluate(
        event: PlayingCard,
        choice: &dyn Choice,
        pot: f64,
        history: &[PlayingCard],
    ) -> Self {
        // create a new history with this card prepended (essentially a backwards history)
        let mut new_history = Vec::with_capacity(history.len() + 1);
        new_history.push(event);
        new_history.extend_from_slice(history);

        // calculate the outcome score for this choice+card
        let new_pot = pot * choice.score(&new_history);
        if new_pot < 1e-6 {
            // we lost (new_pot == 0), so there is no next decision tree
            return Self {
                event,
                value: 0.0,
                next_decision_tree: None,
            };
        }

        // compute the decision tree for the next decision (if it exists)
        let next_decision_tree = choice
            .next_decision()
            .map(|decision| DiscreteDecisionTree::compute(decision, new_pot, &new_history));
        // get the value of this outcome
        // the value is the expected value of the optimal choice of the next decision
        // if there is no next decision, then the value is simply the new_pot
        let value = next_decision_tree
            .as_ref()
            .and_then(|ddt| ddt.optimal())
            .map(|choice| choice.expected_value)
            .unwrap_or(new_pot);
        Self {
            event,
            value,
            next_decision_tree,
        }
    }

    /// The child decision tree for this outcome
    pub fn next_decision(&self) -> Option<&DiscreteDecisionTree> {
        self.next_decision_tree.as_ref()
    }
    /// The count is the total number of outcomes for this event
    ///
    /// If there is no next decision, then it is trivially 1. Otherwise, it's the number
    /// of outcomes in the next decision
    pub fn count(&self) -> usize {
        self.next_decision_tree
            .as_ref()
            .map(|decision| decision.outcomes)
            .unwrap_or(1)
    }
}
