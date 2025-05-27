% be warned, i'm not very good at prolog code and so this can
% probably be improved significantly. I just made it for fun :)
%
% example (or really only) rule to evaluate:
% ?- best_choice_ev(1.0, [], pickColor, BestChoice, BestEV).
% BestChoice = black,
% BestEV = 1.2249773755656113 .

% define card ranks
rank(2).
rank(3).
rank(4).
rank(5).
rank(6).
rank(7).
rank(8).
rank(9).
rank(10).
rank(11). % jack
rank(12). % queen
rank(13). % king
rank(14). % ace
% define card suits
suit(hearts).
suit(diamonds).
suit(spades).
suit(clubs).
% define a card/deck
card(Rank-Suit) :- rank(Rank), suit(Suit).
deck(Cards) :- findall(Card, card(Card), Cards).
% define a rule for remaining cards
remaining_cards(History, Remaining) :-
    deck(Cards),
    subtract(Cards, History, Remaining).

% define choices for each decision
choice(pickColor, red).
choice(pickColor, black).
choice(pickLatitude, higher).
choice(pickLatitude, lower).
choice(pickBounds, inside).
choice(pickBounds, outside).
choice(pickSuit, hearts).
choice(pickSuit, diamonds).
choice(pickSuit, spades).
choice(pickSuit, clubs).
% define next decisions
next_decision(pickColor, pickLatitude).
next_decision(pickLatitude, pickBounds).
next_decision(pickBounds, pickSuit).

% define a score for each choice
% i.e. if Choice == red and Suit == diamonds or hearts, then Score = 2.0
% pickColor red/black 2x
score(red, [_-Suit|_], 2) :-
    (Suit = hearts; Suit = diamonds), !.
score(black, [_-Suit|_], 2) :-
    (Suit = spades; Suit = clubs), !.
% pickLatitude higher/lower 2x->3x
score(higher, [CurRank-_, PrevRank-_|_], Score) :-
    CurRank >= PrevRank,
    !, Score is 3 / 2.
score(lower, [CurRank-_, PrevRank-_|_], Score) :-
    CurRank < PrevRank,
    !, Score is 3 / 2.
% pickBounds inside/outside 3x->4x
score(inside, [CurRank-_, Rank1-_, Rank2-_|_], Score) :-
    min_list([Rank1, Rank2], Low),
    max_list([Rank1, Rank2], High),
    CurRank >= Low, CurRank =< High,
    !, Score is 4 / 3.
score(outside, [CurRank-_, Rank1-_, Rank2-_|_], Score) :-
    min_list([Rank1, Rank2], Low),
    max_list([Rank1, Rank2], High),
    (CurRank < Low; CurRank > High),
    !, Score is 4 / 3.
% pickSuit hearts/diamonds/spades/clubs 4x->10x
score(Suit, [_-Suit|_], Score) :-
    !, Score is 10 / 4.
score(_, _, 0) :- !. % anything that doesn't score is 0

% rule for the EV of the best (or optimal) choice in a decision
% BestEV is unified with the maximum EV of a choice in this decision
% in rust: equal to solver::DiscreteDecisionTree
best_choice_ev(Pot, History, Decision, BestChoice, BestEV) :-
    findall(C, choice(Decision, C), Choices),
    maplist(paired_choice_ev(Pot, History), Choices, Pairs), % unify EV-Choice of choices into Pairs
    % sort the EV-Choice pairs by the key (EV)
    % in addition, add a cashout choice with the Pot being the EV
    keysort([Pot-cashout|Pairs], Sorted),
    reverse(Sorted, [BestEV-BestChoice|_]).
paired_choice_ev(Pot, History, Choice, EV-Choice) :- % helper for above rule
    choice_ev(Pot, History, Choice, EV).

% rule for the EV of a choice
% the EV is unified with the average EVs of all random events
% in rust: equal to solver::ChoiceEval
choice_ev(Pot, History, Choice, EV) :-
    remaining_cards(History, Remaining),
    maplist(random_event_ev(Pot, History, Choice), Remaining, EVs),
    sum_list(EVs, EVSum),
    length(EVs, EVCount),
    EV is EVSum / EVCount.

% rules for the EV of a choice+card
% in rust: equal to solver::RandomEventOutcome
% if a choice+card has a next decision, then the EV is the EV of the optimal choice
random_event_ev(Pot, History, Choice, Card, EV) :-
    choice(ThisDecision, Choice),
    next_decision(ThisDecision, NextDecision),
    NewHistory = [Card|History],
    score(Choice, NewHistory, Score),
    NewPot is Score * Pot,
    NewPot > 0,
    best_choice_ev(NewPot, NewHistory, NextDecision, _, EV).
% if a choice+card doesn't have a next decision, then the EV is just the new pot
random_event_ev(Pot, History, Choice, Card, EV) :-
    score(Choice, [Card|History], Score),
    EV is Score * Pot.
