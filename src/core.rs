use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use once_cell::sync::Lazy;

pub const DECK_SIZE: u8 = 81;
const RANK_BASE: u8 = 3;
const RANK_COLOR: u8 = RANK_BASE.pow(3);
const RANK_COUNT: u8 = RANK_BASE.pow(2);
const RANK_SHADE: u8 = RANK_BASE.pow(1);

#[derive(Debug, PartialEq, Clone)]
pub struct Card(u8);

impl std::ops::Add for Card {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self((self.0 + other.0).rem_euclid(DECK_SIZE))
    }
}

impl std::ops::Sub for Card {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(
            (if self.0 < other.0 {
                self.0 + DECK_SIZE
            } else {
                self.0
            }) - other.0,
        )
    }
}

impl From<CardProperties> for Card {
    fn from(properties: CardProperties) -> Self {
        let CardProperties {
            color,
            count,
            shade,
            shape,
        } = properties;
        Self(
            (color.to_u8().expect("color did not fit in u8") * RANK_COLOR)
                + (count.to_u8().expect("count did not fit in u8") * RANK_COUNT)
                + (shade.to_u8().expect("shade did not fit in u8") * RANK_SHADE)
                + shape.to_u8().expect("shape did not fit in u8"),
        )
    }
}

impl From<Card> for CardProperties {
    fn from(card: Card) -> Self {
        let color = Color::from_u8(card.0 / RANK_COLOR).expect("invalid color enum value");
        let remainder = card.0 % RANK_COLOR;
        let count = Count::from_u8(remainder / RANK_COUNT).expect("invalid count enum value");
        let remainder = remainder % RANK_COUNT;
        let shade = Shade::from_u8(remainder / RANK_SHADE).expect("invalid shade enum value");
        let shape = Shape::from_u8(remainder % RANK_SHADE).expect("invalid shape enum value");
        Self {
            color,
            count,
            shade,
            shape,
        }
    }
}

/// Names drawn from https://en.wikipedia.org/wiki/Set_(card_game)
#[derive(Debug, PartialEq, Clone)]
pub struct CardProperties {
    color: Color,
    count: Count,
    shade: Shade,
    shape: Shape,
}

#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum Color {
    Red = 0,
    Green = 1,
    Purple = 2,
}

#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum Count {
    One = 0,
    Two = 1,
    Three = 2,
}

#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum Shade {
    Solid = 0,
    Striped = 1,
    Open = 2,
}

#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum Shape {
    Diamond = 0,
    Squiggle = 1,
    Oval = 2,
}

/// A complete, ordered deck.
pub static DECK: Lazy<Vec<Card>> = Lazy::new(|| (0..DECK_SIZE).map(Card).collect());

/// A selection of three cards.
pub struct Triple {
    cards: (Card, Card, Card),
}

impl From<(Card, Card, Card)> for Triple {
    fn from(cards: (Card, Card, Card)) -> Self {
        Self { cards }
    }
}

impl Triple {
    /// Return whether the three given cards are a set.
    pub fn is_set(&self) -> bool {
        let (a, b, c) = &self.cards;
        unimplemented!()
    }
}

/// The full game state.
pub struct Table {
    deck: Vec<Card>,
    board: Vec<Card>,
}

impl Table {
    /// Set up a new game, specifying the deck order.
    pub fn new(deck: Vec<Card>) -> Self {
        Self {
            deck,
            board: Vec::new(),
        }
    }

    /// Helper for setting up a fresh board without listing the deck order.
    pub fn new_from_seed(seed: u64) -> Self {
        use rand::{seq::SliceRandom, SeedableRng};
        use rand_pcg::Pcg64;

        let mut deck: Vec<_> = DECK.clone();
        let mut rng = Pcg64::seed_from_u64(seed);
        deck.shuffle(&mut rng);
        Self::new(deck)
    }

    /// Allow an external entity to manipulate the board.
    pub fn board_mut(&mut self) -> &mut Vec<Card> {
        &mut self.board
    }

    /// Deal a single card from the deck.
    pub fn deal(&mut self) -> Option<Card> {
        self.deck.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn card_add() {
        assert_eq!(Card(0) + Card(40), Card(40));
        assert_eq!(Card(40) + Card(40), Card(80));
        assert_eq!(Card(80) + Card(40), Card(39));
        assert_eq!(Card(80) + Card(1), Card(0));
    }

    #[test]
    fn card_sub() {
        assert_eq!(Card(80) - Card(40), Card(40));
        assert_eq!(Card(40) - Card(40), Card(0));
        assert_eq!(Card(0) - Card(40), Card(41));
        assert_eq!(Card(0) - Card(1), Card(80));
    }

    #[test]
    fn card_sub_exhaustive() {
        for a in 0..DECK_SIZE {
            for b in 0..DECK_SIZE {
                let result = std::panic::catch_unwind(|| {
                    let _ = Card(a) - Card(b);
                });
                if result.is_err() {
                    panic!("card subtraction panicked on '{a} - {b}'");
                }
            }
        }
    }

    #[test]
    fn card_into_properties() {
        assert_eq!(
            CardProperties::from(Card(0)),
            CardProperties {
                color: Color::Red,
                count: Count::One,
                shade: Shade::Solid,
                shape: Shape::Diamond
            }
        );
        assert_eq!(
            CardProperties::from(Card(40)),
            CardProperties {
                color: Color::Green,
                count: Count::Two,
                shade: Shade::Striped,
                shape: Shape::Squiggle
            }
        );
        assert_eq!(
            CardProperties::from(Card(80)),
            CardProperties {
                color: Color::Purple,
                count: Count::Three,
                shade: Shade::Open,
                shape: Shape::Oval
            }
        );
    }

    #[test]
    fn card_properties_roundtrip_exhaustive() {
        for card in DECK.iter() {
            let result = std::panic::catch_unwind(|| {
                assert_eq!(Card::from(CardProperties::from(card.clone())), card.clone());
            });
            if result.is_err() {
                panic!("card properties roundtrip panicked on '{card:?}'");
            }
        }
    }

    /*
        const RED_ONE_SOLID_DIAMOND: Card = Card {
            color: Color::Red,
            count: Count::One,
            shade: Shade::Solid,
            shape: Shape::Diamond,
        };
        const GREEN_TWO_SOLID_DIAMOND: Card = Card {
            color: Color::Green,
            count: Count::Two,
            shade: Shade::Solid,
            shape: Shape::Diamond,
        };
        const PURPLE_THREE_SOLID_DIAMOND: Card = Card {
            color: Color::Purple,
            count: Count::Three,
            shade: Shade::Solid,
            shape: Shape::Diamond,
        };
        const GREEN_TWO_STRIPED_OVAL: Card = Card {
            color: Color::Green,
            count: Count::Two,
            shade: Shade::Striped,
            shape: Shape::Oval,
        };
        const PURPLE_THREE_OPEN_SQUIGGLE: Card = Card {
            color: Color::Purple,
            count: Count::Three,
            shade: Shade::Open,
            shape: Shape::Squiggle,
        };

        #[test]
        fn is_set_works() {
            // All the same card is tecnically a set
            assert_eq!(
                Triple::from((
                    RED_ONE_SOLID_DIAMOND.clone(),
                    RED_ONE_SOLID_DIAMOND.clone(),
                    RED_ONE_SOLID_DIAMOND.clone(),
                ))
                .is_set(),
                true
            );
            // A mixed set
            assert_eq!(
                Triple::from((
                    RED_ONE_SOLID_DIAMOND.clone(),
                    GREEN_TWO_SOLID_DIAMOND.clone(),
                    PURPLE_THREE_SOLID_DIAMOND.clone(),
                ))
                .is_set(),
                true
            );
            // All different set
            assert_eq!(
                Triple::from((
                    RED_ONE_SOLID_DIAMOND.clone(),
                    GREEN_TWO_STRIPED_OVAL.clone(),
                    PURPLE_THREE_OPEN_SQUIGGLE.clone(),
                ))
                .is_set(),
                true
            );
            // Not sets
            assert_eq!(
                Triple::from((
                    RED_ONE_SOLID_DIAMOND.clone(),
                    RED_ONE_SOLID_DIAMOND.clone(),
                    PURPLE_THREE_SOLID_DIAMOND.clone(),
                ))
                .is_set(),
                false
            );
            assert_eq!(
                Triple::from((
                    RED_ONE_SOLID_DIAMOND.clone(),
                    GREEN_TWO_SOLID_DIAMOND.clone(),
                    PURPLE_THREE_OPEN_SQUIGGLE.clone(),
                ))
                .is_set(),
                false
            );
        }
    */
}
