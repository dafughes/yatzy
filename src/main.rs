use std::collections::{HashMap, HashSet};

use leptos::{logging, prelude::*};

const BONUS_THRESHOLD: u32 = 63;
const BONUS_SCORE: u32 = 35;
const FULL_HOUSE_SCORE: u32 = 25;
const SMALL_STRAIGHT_SCORE: u32 = 30;
const LARGE_STRAIGHT_SCORE: u32 = 40;
const YATZY_SCORE: u32 = 50;

const NUM_DICE: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rule {
    Pips(u32),
    Bonus,
    Count(usize),
    FullHouse,
    SmallStraight,
    LargeStraight,
    Yatzy,
    Chance,
}

impl Rule {
    pub fn list_upper() -> Vec<Self> {
        vec![
            Self::Pips(1),
            Self::Pips(2),
            Self::Pips(3),
            Self::Pips(4),
            Self::Pips(5),
            Self::Pips(6),
        ]
    }

    pub fn list_all() -> Vec<Self> {
        let upper = Self::list_upper();
        let lower = Self::list_lower();

        [&upper[..], &[Rule::Bonus], &lower[..]].concat()
    }

    pub fn list_lower() -> Vec<Self> {
        vec![
            Self::Count(3),
            Self::Count(4),
            Self::FullHouse,
            Self::SmallStraight,
            Self::LargeStraight,
            Self::Yatzy,
            Self::Chance,
        ]
    }

    pub fn score(self, dice: &[u32]) -> u32 {
        let count = |i| dice.iter().filter(|&x| *x == i).count();
        let sum = dice.iter().sum();
        let set: HashSet<_> = dice.iter().copied().collect();

        match self {
            Self::Pips(n) => count(n) as u32 * n,
            Self::Bonus => 0,
            Self::Count(n) => (1..=6)
                .any(|i| count(i) >= n)
                .then_some(sum)
                .unwrap_or_default(),
            Self::FullHouse => {
                let pair = set.iter().find(|&&i| count(i) == 2);
                let triple = set.iter().find(|&&i| count(i) == 3);

                if pair.is_some() && triple.is_some() {
                    FULL_HOUSE_SCORE
                } else {
                    0
                }
            }
            Self::SmallStraight => [1..=4, 2..=5, 3..=6]
                .iter()
                .any(|r| r.clone().all(|i| set.contains(&i)))
                .then(|| SMALL_STRAIGHT_SCORE)
                .unwrap_or_default(),
            Self::LargeStraight => [1..=5, 2..=6]
                .iter()
                .any(|r| r.clone().all(|i| set.contains(&i)))
                .then(|| LARGE_STRAIGHT_SCORE)
                .unwrap_or_default(),
            Self::Yatzy => {
                if set.len() == 1 {
                    YATZY_SCORE
                } else {
                    0
                }
            }
            Self::Chance => sum,
        }
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Rule::Pips(1) => "Ones",
            Rule::Pips(2) => "Twos",
            Rule::Pips(3) => "Threes",
            Rule::Pips(4) => "Fours",
            Rule::Pips(5) => "Fives",
            Rule::Pips(6) => "Sixes",
            Rule::Bonus => "Bonus",
            Rule::Count(3) => "Three of a kind",
            Rule::Count(4) => "Four of a kind",
            Rule::FullHouse => "Full house",
            Rule::SmallStraight => "Small straight",
            Rule::LargeStraight => "Large straight",
            Rule::Yatzy => "Yatzy",
            Rule::Chance => "Chance",
            _ => unreachable!(),
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct Player {
    name: String,
    scores: HashMap<Rule, u32>,
}

impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            scores: HashMap::new(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get(&self, rule: Rule) -> Option<u32> {
        self.scores.get(&rule).copied()
    }

    pub fn set(&mut self, rule: &Rule, score: u32) {
        self.scores.entry(*rule).or_insert(score);

        let no_bonus = Rule::list_upper()
            .iter()
            .all(|rule| self.scores.get(rule).is_some());

        if self.bonus_progress() >= BONUS_THRESHOLD {
            self.scores.insert(Rule::Bonus, BONUS_SCORE);
        } else if no_bonus {
            self.scores.insert(Rule::Bonus, 0);
        }
    }

    pub fn total(&self) -> u32 {
        self.scores.values().sum()
    }

    pub fn bonus_progress(&self) -> u32 {
        Rule::list_upper()
            .iter()
            .filter_map(|rule| self.scores.get(rule))
            .sum()
    }
}

#[component]
fn ScoreView(
    players: RwSignal<Vec<Player>>,
    dice: RwSignal<Vec<u32>>,
    locked_dice: RwSignal<Vec<bool>>,
    roll_count: RwSignal<usize>,
    current_player: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div id="scores">
            <table>
                <thead>
                    <tr>
                        <th class="rule" />
                        {
                            players.with_untracked(|players| players.iter().map(|player| view!{
                                <th>{player.name()}</th>
                            }).collect_view())
                        }
                    </tr>
                </thead>
                <tbody>
                    {
                        Rule::list_all().into_iter().map(|rule| view! {
                            <tr>
                                <td class="rule">{rule.to_string()}</td>
                                {
                                    move || players.get().into_iter().enumerate().map(|(j, player)| {
                                        let has_rolled = roll_count.get() > 0;
                                        let is_current_player = current_player.get() == j;
                                        let is_bonus = matches!(rule, Rule::Bonus);
                                        let is_cell_empty = player.get(rule).is_none();

                                        // Score with joker rule
                                        let score = {
                                            let d = dice.get();
                                            let has_yatzy = player.get(Rule::Yatzy).is_some();
                                            if Rule::Yatzy.score(&d) > 0 && has_yatzy {
                                                // Extra yatzies can be recorded in any cell
                                                YATZY_SCORE
                                            } else {
                                                rule.score(&d)
                                            }
                                        };

                                        // TODO: clean this
                                        let score_str = move || match player.get(rule) {
                                            Some(score) => score.to_string(),
                                            _ if has_rolled && is_current_player && !is_bonus => score.to_string(),
                                            _ => String::new()
                                        };

                                        let is_preview = move || is_current_player && is_cell_empty && has_rolled && !is_bonus;

                                        let on_click = move |_| {
                                            // Make sure only cells that are empty and belong to current player can be clicked
                                            // Can't also set bonus directly
                                            if has_rolled && is_current_player && !is_bonus && is_cell_empty {
                                                players.update(|players| players[j].set(&rule, score));

                                                current_player.set((current_player.get() + 1) % players.read().len());
                                                roll_count.set(0);
                                                locked_dice.set(vec![false; NUM_DICE]);
                                            }
                                        };

                                        view!{
                                            <td class:preview=is_preview on:click=on_click>{score_str}</td>
                                        }
                                    }).collect_view()
                                }



                            </tr>
                        }).collect_view()
                    }
                </tbody>
                <tfoot>
                    <tr>
                        <td class="rule">"Total"</td>
                        {
                            move || players.read().iter().map(|player| view!{
                                <td>{player.total()}</td>
                            }).collect_view()
                        }
                    </tr>
                </tfoot>
            </table>
        </div>
    }
}

#[component]
fn DiceView(
    dice: RwSignal<Vec<u32>>,
    locked_dice: RwSignal<Vec<bool>>,
    roll_count: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div id="dice">
            {
                move || dice.get().iter().enumerate().map(|(i, die)| {
                    let has_rolled = roll_count.get() > 0;
                    let can_lock = has_rolled && roll_count.get() < 3;

                    let dice_str = has_rolled.then_some(format!("images/{die}.svg")).unwrap_or(String::from(""));

                    let is_locked = locked_dice.read()[i];
                    let on_click = move |_| locked_dice.update(|locked| if can_lock {locked[i] = !locked[i]});

                    view! {
                        <div class="die" class:locked=is_locked>
                            <img src=dice_str on:click=on_click/>
                        </div>
                    }
                }).collect_view()
            }
        </div>
        <div id="roll">
            {
                move || {
                    let is_disabled = move || roll_count.get() >= 3;
                    let on_click = move |_| {
                        let new_dice =
                            dice.get()
                            .iter()
                            .zip(locked_dice.get())
                            .map(|(&die, locked)| locked.then_some(die).unwrap_or(rand::random_range(1..=6))).collect();

                        dice.set(new_dice);
                        roll_count.set(roll_count.get() + 1);
                        locked_dice.set(vec![false; NUM_DICE]);
                    };

                    view! {
                        <button disabled=is_disabled on:click=on_click>"ROLL"</button>
                    }
                }
            }
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let players = RwSignal::new(vec![
        Player::new("1"),
        Player::new("2"),
        // Player::new("3"),
        // Player::new("4"),
    ]);

    let dice = RwSignal::new(vec![1; NUM_DICE]);
    let locked_dice = RwSignal::new(vec![false; NUM_DICE]);
    let roll_count = RwSignal::new(0);
    let current_player = RwSignal::new(0);

    view! {
        <ScoreView players dice locked_dice roll_count current_player />
        <DiceView dice locked_dice roll_count/>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| App);
}
