<div align="center">

[![Pleco][pleco-badge]][pleco-link]

[![Build][build-badge]][build-link]
[![License][license-badge]][license-link]
[![Commits][commits-badge]][commits-link]

</div>

# Overview

Pleco is a Chess Library, containing the building blocks of the chess engine [Stockfish](https://stockfishchess.org/),
re-written entirely in Rust.

This project is split into two crates, `pleco` (the library you are currently in), which contains the library functionality,
and `pleco_engine`, which contains the UCI (Universal Chess Interface) compatible Engine & AI.

The overall goal of pleco is to recreate the Stockfish engine in rust, for comparison and
educational purposes. As such, the majority of the algorithms used here are a direct port of Stockfish's, and the
credit for the majority of the code go directly to the maintainers and authors of Stockfish.

- [Documentation](https://docs.rs/pleco)
- [crates.io](https://crates.io/crates/pleco)

For the chess engine implemented using this library provided by `pleco`,
see [pleco_engine](https://github.com/sfleischman105/Pleco/tree/main/pleco_engine).

## Features

Some of the features `pleco` implements:

- Bitboard Representation of Piece Locations:
- Ability for concurrent Board State access, for use by parallel searchers
- Full Move-generation Capabilities, including generation of pseudo-legal moves
- Statically computed lookup-tables (including Magic Bitboards)
- Zobrist Hashing
- A Transposition Table: lightning fast lookup table storing information for a board
- Pre-implemented searchers, some of which using [rayon.rs](https://crates.io/crates/rayon) for easy parallelism

## Use

To use Pleco inside your own Rust projects,
[Pleco.rs is available as a library on crates.io](https://crates.io/crates/pleco).
Pleco runs on all three distributions (`nightly`, `beta`, `stable`) of rust.

### Basic Usage

Setting up a board position is extremely simple.

```rust
use pleco::{Board,Player,PieceType};

let board = Board::start_pos();
assert_eq!(board.count_piece(Player::White,PieceType::P), 8);
assert_eq!(&board.fen(),"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
```

#### Creating a board from a Position

A `Board` can be created with any valid chess position using a valid FEN (Forsyth-Edwards Notation) String.
Check out the [Wikipedia article](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) for more information on FEN Strings
and their format.

```rust
let board: Board = Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2").unwrap();
```

#### Applying and Generating Moves

Moves are represented with a `BitMove` structure. They must be generated by a `Board` object directly, to be
considered a valid move. Using `Board::generate_moves()` will generate all legal `BitMove`s of the current
position for the current player.

```rust
use pleco::{Board,BitMove};

let mut board = Board::start_pos(); // create a board of the starting position
let moves = board.generate_moves(); // generate all possible legal moves
board.apply_move(moves[0]);
assert_eq!(board.moves_played(), 1);
```

We can ask the Board to apply a move to itself from a string. This string must follow the format of a standard
UCI Move, in the format [src_sq][dst_sq][promo]. E.g., moving a piece from A1 to B3 would have a uci string of "a1b3",
while promoting a pawn would look something like "e7e81". If the board is supplied a UCI move that is either
incorrectly formatted or illegal, false shall be returned.

```rust
let mut board = Board::start_pos(); // create a board of the starting position
let success = board.apply_uci_move("e7e8q"); // apply a move where piece on e7 -> eq, promotes to queen
assert!(!success); // Wrong, not a valid move for the starting position
```

#### Undoing Moves

We can revert to the previous chessboard state with a simple `Board::undo_move()`:

```rust
let mut board = Board::start_pos();
board.apply_uci_move("e2e4"); // A very good starting move, might I say
assert_eq!(board.moves_played(),1);
board.undo_move();
assert_eq!(board.moves_played(),0);
```

## Contributing

Any and all contributions are welcome! Open up a PR to contribute some improvements. Look at the Issues tab to see what needs some help.

## License

Pleco is distributed under the terms of the MIT license. See LICENSE-MIT for details. Opening a pull requests is assumed to signal agreement with these licensing terms.

[build-link]: https://github.com/pleco-rs/Pleco/blob/main/.github/workflows/test.yml
[build-badge]: https://img.shields.io/github/workflow/status/pleco-rs/Pleco/Test?style=for-the-badge&label=tanton&logo=github
[license-badge]: https://img.shields.io/github/license/pleco-rs/Pleco?style=for-the-badge&label=license&color=success
[license-link]: https://github.com/pleco-rs/Pleco/blob/main/LICENSE
[commits-badge]: https://img.shields.io/github/commit-activity/m/pleco-rs/Pleco?style=for-the-badge
[commits-link]: https://github.com/pleco-rs/Pleco/commits/main
[pleco-badge]: https://img.shields.io/crates/v/pleco.svg?style=for-the-badge
[pleco-link]: https://crates.io/crates/pleco
[pleco-engine-badge]: https://img.shields.io/crates/v/pleco_engine.svg?style=for-the-badge
[pleco-engine-link]: https://crates.io/crates/pleco_engine
