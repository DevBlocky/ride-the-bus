mod card;
mod decision;
use std::{io, str::FromStr};

use card::PlayingCard;
use decision::{
    Choice, DiscreteDecision,
    solver::{DiscreteDecisionTree, EvalChoice, RandomEventOutcome},
};

#[derive(Debug)]
enum PickColor {
    Red,
    Black,
}
impl Choice for PickColor {
    fn score(&self, cards: &[PlayingCard]) -> f64 {
        match (self, cards[0].color()) {
            (Self::Red, 0) | (Self::Black, 1) => 2.0, // 1x -> 2x
            (_, 0..=1) => 0.0,
            _ => unreachable!(),
        }
    }
    fn next_decision(&self) -> DiscreteDecision {
        DiscreteDecision::new_with_cashout([PickLatitude::Higher, PickLatitude::Lower])
    }
}
#[derive(Debug)]
enum PickLatitude {
    Higher,
    Lower,
}
impl Choice for PickLatitude {
    fn score(&self, cards: &[PlayingCard]) -> f64 {
        match (self, cards[0].rank() >= cards[1].rank()) {
            (Self::Higher, true) | (Self::Lower, false) => 3.0 / 2.0, // 2x -> 3x
            (Self::Higher, false) | (Self::Lower, true) => 0.0,
        }
    }
    fn next_decision(&self) -> DiscreteDecision {
        DiscreteDecision::new_with_cashout([PickContained::Inside, PickContained::Outside])
    }
}
#[derive(Debug)]
enum PickContained {
    Inside,
    Outside,
}
impl Choice for PickContained {
    fn score(&self, cards: &[PlayingCard]) -> f64 {
        let c1 = cards[1].rank(); // last card seen
        let c2 = cards[2].rank(); // 2nd last card seen
        let bounds = u8::min(c1, c2)..=u8::max(c1, c2);
        match (self, bounds.contains(&cards[0].rank())) {
            (Self::Inside, true) | (Self::Outside, false) => 4.0 / 3.0, // 3x -> 4x
            (Self::Inside, false) | (Self::Outside, true) => 0.0,
        }
    }
    fn next_decision(&self) -> DiscreteDecision {
        DiscreteDecision::new_with_cashout([
            PickSuit::Hearts,
            PickSuit::Diamonds,
            PickSuit::Spades,
            PickSuit::Clubs,
        ])
    }
}
#[derive(Debug)]
enum PickSuit {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}
impl Choice for PickSuit {
    fn score(&self, cards: &[PlayingCard]) -> f64 {
        match (self, cards[0].suit()) {
            (Self::Hearts, 0) | (Self::Diamonds, 1) | (Self::Spades, 2) | (Self::Clubs, 3) => {
                10.0 / 4.0 // 4x -> 10x
            }
            (_, 0..=3) => 0.0,
            _ => unreachable!(),
        }
    }
    fn next_decision(&self) -> DiscreteDecision {
        DiscreteDecision::empty()
    }
}

struct InvalidCommandErr;
enum Command {
    Help,
    Exit,
    ListChoices,
    ListEvents(String),

    Reset,
    Back,
    Card(PlayingCard),
}
impl FromStr for Command {
    type Err = InvalidCommandErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim().split(" ");
        match split.next() {
            Some("help") => Ok(Command::Help),
            Some("exit") => Ok(Command::Exit),
            Some("list") => Ok(split
                .next()
                .map(ToOwned::to_owned)
                .map(Command::ListEvents)
                .unwrap_or(Command::ListChoices)),
            Some("reset") => Ok(Command::Reset),
            Some("back") => Ok(Command::Back),
            Some(cmd) => PlayingCard::from_str(cmd)
                .map(Command::Card)
                .map_err(|_| InvalidCommandErr),
            _ => Err(InvalidCommandErr),
        }
    }
}
impl Command {
    fn from_stdin() -> io::Result<Self> {
        loop {
            print!("? ");
            io::Write::flush(&mut io::stdout())?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;

            if let Ok(cmd) = Command::from_str(&line) {
                return Ok(cmd);
            }
            println!("invalid command");
        }
    }
}

fn print_help() {
    println!("\n[Commands]");
    println!("help = This command");
    println!("exit = Quit the program");
    println!("list = Prints the choices and the expected values");
    println!("list {{choice_name|'optimal'}} = Prints the random events associated with a choice");
    println!("reset = Start over (new game)");
    println!("back = Go back to previous choice (useful if you input the wrong card)");
    println!("{{card}} = Input a card (your choice can be interpreted)");

    println!("\n[Card Format]");
    println!(
        "Card formats are pretty simple. It's the value (or letter) of the card, plus the suit, case insensitive"
    );
    println!("Examples:");
    println!("2H  = 2 of hearts");
    println!("10C = 10 of clubs");
    println!("QD  = Queen of diamonds");
    println!("AS  = Ace of spades");

    println!("\n[Tutorial]");
    println!(
        "This tool is intended to be used while playing Ride The Bus (a fictional casino game) in Schedule I"
    );
    println!(
        "1. You are shown a series of Choices and their Expected Values. In Schedule I, choose the option with the highest indicated expected value here"
    );
    println!(
        "2. Once your option is selected, a dealer then places another card in front of you. Input the shown card here using the CLI"
    );
    println!(
        "3. After inputting your card, your choice is automatically interpreted and a new series of choices is shown"
    );
    println!("4. Repeat Step 1-3 until you either lose or cashout, then restart with '? reset'")
}
fn print_choices(tree: &DiscreteDecisionTree) {
    println!("[Choices]");
    println!("# Choice = Expected Value");
    // get the EV for the optimal choice, used to show an arrow to the best choices (ones equalling this EV)
    let optimal_ev = tree.optimal().map(|x| x.expected_value).unwrap_or(0.0);
    for choice in tree.choices() {
        print!("{:?} = {:.04}", choice.choice, choice.expected_value);
        if choice.expected_value >= (optimal_ev - 1e-6) {
            println!(" <----");
        } else {
            println!();
        }
    }
}
fn print_events(tree: &DiscreteDecisionTree, choice_name: &str) {
    // find an option to the target to enumerate for this command
    let list_target = match choice_name {
        "optimal" => tree.optimal(),
        name => tree
            .choices()
            .iter()
            .find(|ec| format!("{:?}", ec.choice) == name),
    };
    // either print the cards and their expected values, or say its an invalid target
    if let Some(target) = list_target {
        println!("[{:?}]", target.choice);
        println!("# REvent = Expected Value");
        for (card, outcome) in target.iter() {
            // only print cards that are winners (EV>0)
            if outcome.expected_value() > 1e-6 {
                println!("{} = {:.04}", card, outcome.expected_value());
            }
        }
    } else {
        println!("invalid list target")
    }
}
fn interactive_prompt(tree: &DiscreteDecisionTree) {
    let mut history = vec![tree];
    'outer: loop {
        // get the current decision tree and print the choices available to the user
        let tree = history.last().expect("non-empty history");
        print_choices(tree);

        // find the next card from user input (service the CLI prompt)
        let next_card = loop {
            let cmd = Command::from_stdin().expect("stdin command");
            match cmd {
                Command::Help => print_help(),
                Command::Exit => std::process::exit(0),
                Command::ListChoices => print_choices(tree),
                Command::ListEvents(choice_name) => print_events(tree, &choice_name),
                Command::Reset => return, // reset to root tree
                Command::Back => {
                    // remove the last taken decision, then restart interaction
                    history.remove(history.len() - 1);
                    continue 'outer;
                }
                Command::Card(card) => break card, // break out with provided card to enter new tree
            }
        };

        // find the choice the user made by finding the max EV
        // we can do this because the decisions are disjoint (except Cashout, which is always smaller), i.e.
        // a card can only succeed with one decision
        let choice: Option<&EvalChoice> = tree.choices().iter().max_by(|c1, c2| {
            let ev1 = c1.get(next_card).map(|o| o.expected_value()).unwrap_or(0.0);
            let ev2 = c2.get(next_card).map(|o| o.expected_value()).unwrap_or(0.0);
            f64::total_cmp(&ev1, &ev2)
        });

        // get the next tree from the card provided, or error if it was an invalid card, or reset
        // if no more decisions are needed (caller is looping us)
        println!();
        let outcome = choice
            .inspect(|c| println!("??? So you chose {:?} ???", c.choice))
            .and_then(|c| c.get(next_card));
        match outcome {
            Some(RandomEventOutcome::Child {
                expected_value: _,
                child,
            }) => history.push(child),
            None => println!("!!! INVALID CARD PROVIDED !!!"),
            _ => break 'outer,
        }
    }
    println!("no more decisions, resetting");
}
fn main() {
    // solve ride the bus
    // this only takes a few seconds, hence why it's fine we do this on every start instead of memoizing
    println!("solving ride the bus");
    let first_decision = DiscreteDecision::new_with_cashout([PickColor::Red, PickColor::Black]);
    let tree = DiscreteDecisionTree::solve(first_decision);
    println!("all games considered, done!");

    // print the tutorial, then start the interactive loop
    print_help();
    loop {
        println!();
        interactive_prompt(&tree);
    }
}
