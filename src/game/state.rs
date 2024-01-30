/// Enum representing the side to move.
#[derive(Clone, Copy, Debug)]
pub enum Side {
    Black,
    White,
}

impl Side {
    fn flip(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

impl Default for Side {
    fn default() -> Self {
        Self::Black
    }
}

/// Enum representing the type of a move (either pass or play).
#[derive(Clone, Copy, Debug)]
pub enum MoveType {
    Pass,
    Play,
}

impl Default for MoveType {
    fn default() -> Self {
        Self::Play
    }
}

/// Struct representing the game state, excluding the board data.
#[derive(Clone, Copy, Debug, Default)]
pub struct GameState {
    side: Side,
    last: MoveType,
}

impl GameState {
    pub fn play(self) -> Self {
        Self { side: self.side.flip(), last: MoveType::Play }
    }

    pub fn pass(self) -> Self {
        Self { side: self.side.flip(), last: MoveType::Pass }
    }

    pub fn get_side(self) -> Side {
        self.side
    }

    pub fn get_last(self) -> MoveType {
        self.last
    }
}