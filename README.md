# Ride The Bus

This program solves Schedule I's Ride The Bus casino game. It provides a CLI
application intended to be used while playing the casino game that indicates
which decision should be made at each step of the game to ensure maximum
profits (highest expected value).

## Principle

This program works by considering every single game of Ride The Bus. Since the game
itself is very simple, and there are only 4 decisions, we can essentially enumerate
the cartesian product of each choice and each card. By knowing the EV of the child
decision, we can know the EV of a card+choice, then we can calculate the EV for a choice
by averaging all card+choice EVs. We do this up the tree of choices until we have solved
the entire game.

## Observations / Cheatsheet

Ride The Bus's expected value is `1.2250` if played optimally. This means that for
every dollar you put into the game, you expect to make 22.5 cents back.

Since the maximum play for Ride The Bus in Schedule I is $500, on average you expect
to profit $112.50 for each play. To profit $10,000, you would expect to play 89 times.

### Pick Red/Black

***TL;DR**: Just pick either*

Obviously, you can pick either red or black and have the same odds since it's
essentially a 50/50 chance.

However, more interestingly, you can also reason that this game must have an
expected value of >=1.0 from just this decision. This step doubles your money 50%
of the time, so it's obviously =1.0 for just this step, and since you can Cashout
you always expect to at least make your money back if played optimally.

### Pick Higher/Lower

***TL;DR**: Cashout on 7,8,9,10, otherwise use intuition*

On this option, you have to pick whether the unseen card will be
higher or lower than the last card shown.

With some intuition, you can pretty much figure out the odds. If the shown card is
high, go lower; low, go higher. Near the middle of the ranks, the odds start
becoming 50/50 again, and since the profits only go from 2x->3x, you should
Cashout.

This program calculates that cards 7, 8, 9, and 10 have expected values below 3.0
for either inside/outside, so you should Cashout if you see one of these cards.

### Pick Inside/Outside

***TL;DR**: Pick outside for A-B<=2, inside for A-B>=9, otherwise Cashout*
(A-B meaning the smaller rank minus the bigger rank)

For this option, you choose whether the unseen card's rank is inside or outside of 
the two seen cards. Inside is inclusive, meaning that if the unseen card is either A
or B, then it is considered inside.

This option was the least intuitive to me, I thought that you would want pick
Inside/Outside unless the range covers close to half of the cards. According to
this program, however, you expect to profit by picking Inside/Outside when the
range is either very small or very large. For example, 2..4's optimal choice is
Outside, but 2..5's optimal choice is Cashout.

More concretely, a range of <=2 cards should be picked as Outside (e.g. 8..10),
\>=9 should be picked as Inside (e.g. 3..Q), and all other cases should Cashout.
This means, most of the time, the best bet is to Cashout.

With some more thought, it's apparent why you would want to Cashout most of the time.
The profits for this choice only go from 3x->4x, and so you would like to have very
good odds for either Inside/Outside to be able to make more money on average than
simply cashingout, else you risk losing the entire bet.

### Pick Suit

**TL;DR: Always Cashout**

This one is much more obvious, but I wish I realized it sooner.

At first, it seems great to try for 10x profits, but because this choice has
roughly a 1/4 chance for each suit, you end up losing your money 3/4 of the time
and only multiplying your money by x2.5 otherwise.

Only if this choice had quadrupled your profits (i.e. it went from 4x->16x), then
choosing a suit you haven't seen would have a slightly higher expected value than
Cashout (thanks to sampling w/o replacement).

## Building/Running

To start, you must have [rust installed](https://www.rust-lang.org/tools/install).

Afterwards, you can either build an executable or run it directly from `cargo`:
```sh
# run the program
cargo run --release

# build the program
cargo build --release
# executable found in ./target/release
```
