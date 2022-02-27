# A Solver for Set (the card game)

## What is Set?

## Playing Set with Engineers

## Let's write a solver!

### Getting started

I chose to write this solver in Rust - it's performant, clear, and most importantly, I like writing Rust code.

Let's go ahead and set up the basics in a new project:

```rust
/// Names drawn from https://en.wikipedia.org/wiki/Set_(card_game)
struct Card {
    color: Color,
    count: Count,
    shade: Shade,
    shape: Shape,
}

enum Color {
    Red,
    Green,
    Purple,
}

enum Count {
    One,
    Two,
    Three,
}

enum Shade {
    Solid,
    Striped,
    Open,
}

enum Shape {
    Diamond,
    Squiggle,
    Oval,
}
```

Then, let's think about the semantics we need, in order to write a solver:

```rust
/// A selection of three cards.
struct Triple {
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
        unimplemented!()
    }
}

/// The full game state.
struct Table {
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
        unimplemented!()
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
```

These methods are very vague, but they allow an external caller to change the state of the board, and receive new cards from the deck.

Note that dealing does not alter the state of the board. This will be important later - if we keep track of data by index reference to the board's `Vec<Card>`, we don't want to alter it outside of the solver's knowledge.

One thing we haven't done yet is specify what our starting deck of cards contains. We know that it contains one card for each combination of properties - we could enumerate them! But let's not - we can use the `strum` crate instead, to turn our enums into iterators:

```rust
use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// the same annotations also applied to our other enums above
#[derive(EnumIter, Debug, PartialEq, Clone, Copy)]
enum Color {
    Red,
    Green,
    Purple,
}

static DECK: Lazy<Vec<Card>> = Lazy::new(|| {
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
```

### Filling in the details

It's now trivial to implement our `new_from_seed` method we specified earlier:

```rust
/// Helper for setting up a fresh board without listing the deck order.
pub fn new_from_seed(seed: u64) -> Self {
    use rand::{seq::SliceRandom, SeedableRng};
    use rand_pcg::Pcg64;

    let mut deck = DECK.clone();
    let mut rng = Pcg64::seed_from_u64(seed);
    deck.shuffle(&mut rng);
    Self::new(deck)
}
```

The final piece of the puzzle is to implement the rules of... what is a set?

```rust
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
```

Huh. That's... not pretty. I already have a better idea, but let's stick with what we've got for now.
It looks fine, but let's write our first test and check:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn is_set_works() {
        // A mixed set
        assert_eq!(
            Triple::from((
                Card {
                    color: Color::Red,
                    count: Count::One,
                    shade: Shade::Solid,
                    shape: Shape::Diamond
                },
                Card {
                    color: Color::Green,
                    count: Count::Two,
                    shade: Shade::Solid,
                    shape: Shape::Diamond
                },
                Card {
                    color: Color::Purple,
                    count: Count::Three,
                    shade: Shade::Solid,
                    shape: Shape::Diamond
                }
            ))
            .is_set(),
            true
        );
    }
}
```

Wow, this is also super wordy. Constant time!

As our `Card` doesn't require any form of allocation, these can be proper structs, not `once_cell`'s `Lazy`'s:

```rust
const RED_ONE_SOLID_DIAMOND: Card = Card {
    color: Color::Red,
    count: Count::One,
    shade: Shade::Solid,
    shape: Shape::Diamond,
};
// etc

#[test]
fn is_set_works() {
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
    // etc
}
```

One final thing - let's shove this in a `core` module, so we don't tangle up the internals in our solver implementation:

```bash
mv lib.rs core.rs
```

```rust
// lib.rs
mod core;
```

### So what was that better idea?

This is already getting pretty wordy and unwieldy, for what _feels_ like should be a simple solution.

Let's stop and think for a moment.

#### Abstract math interlude

#### So, let's change the internals

First off, we can define our deck as a simple vector of integers:

```rust
#[derive(Debug, PartialEq, Clone)]
pub struct Card(u8);

/// A complete, ordered deck.
pub static DECK: Lazy<Vec<Card>> = Lazy::new(|| (0..81).map(Card).collect());
```

This allows us to remove `strum` and the `EnumIter` stuff we needed earlier.

We can rename our original struct `CardProperties`. We should also allow converting from `Card` to `CardProperties` and back, as they're equivalent:

```rust
const RANK_BASE: u8 = 3;
const RANK_COLOR: u8 = RANK_BASE.pow(3);
const RANK_COUNT: u8 = RANK_BASE.pow(2);
const RANK_SHADE: u8 = RANK_BASE.pow(1);

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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
}
```

We're also going to need to do some basic math with cards, so let's get that sorted:

```rust
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
                    panic!("card subtraction paniced on '{a} - {b}'");
                }
            }
        }
    }
}
```

We can now happily redefine what a set is!

```rust
impl Triple {
    /// Return whether the three given cards are a set.
    pub fn is_set(&self) -> bool {
        let (a, b, c) = &self.cards;
        let mut cards = [a, b, c];
        cards.sort();
        let [a, b, c] = cards;
        *a - *b == *b - *c
    }
}
```

Simple!

Let's more on to actually solving this problem we've created.

### A brief aside: visualisation

This math problem we've constructed is based on a visual game. Naturally, it might help a little to add some visualisation to our data.
