use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Names drawn from https://en.wikipedia.org/wiki/Set_(card_game)
#[derive(Debug, PartialEq, Clone)]
pub struct Card {
    color: Color,
    count: Count,
    shade: Shade,
    shape: Shape,
}

#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Purple,
}

#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
pub enum Count {
    One,
    Two,
    Three,
}

#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
pub enum Shade {
    Solid,
    Striped,
    Open,
}

#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
pub enum Shape {
    Diamond,
    Squiggle,
    Oval,
}

/// A complete, ordered deck.
pub static DECK: Lazy<Vec<Card>> = Lazy::new(|| {
    let mut deck = Vec::with_capacity(81);
    for color in Color::iter() {
        for count in Count::iter() {
            for shade in Shade::iter() {
                for shape in Shape::iter() {
                    deck.push(Card {
                        color,
                        count,
                        shade,
                        shape,
                    });
                }
            }
        }
    }
    deck
});

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
        if !((a.color == b.color && b.color == c.color)
            || (a.color != b.color && b.color != c.color && c.color != a.color))
        {
            return false;
        }
        if !((a.count == b.count && b.count == c.count)
            || (a.count != b.count && b.count != c.count && c.count != a.count))
        {
            return false;
        }
        if !((a.shade == b.shade && b.shade == c.shade)
            || (a.shade != b.shade && b.shade != c.shade && c.shade != a.shade))
        {
            return false;
        }
        if !((a.shape == b.shape && b.shape == c.shape)
            || (a.shape != b.shape && b.shape != c.shape && c.shape != a.shape))
        {
            return false;
        }
        true
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

        let mut deck = DECK.clone();
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
}
