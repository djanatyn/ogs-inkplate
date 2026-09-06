use ogs_fetch::compression::{compress_games_from_directory, generate_c_header, parse_sgf_moves};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BOARD_SIZE: usize = 19;
const EMPTY: u8 = 0;
const BLACK: u8 = 1;
const WHITE: u8 = 2;
const PASS_MOVE: u16 = 0xFFFF;

type Board = [[u8; BOARD_SIZE]; BOARD_SIZE];

fn fixture_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(root.join("tests/fixtures/sgf"))
        .expect("missing SGF fixtures")
        .map(|entry| entry.expect("failed to read fixture entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sgf"))
        .collect::<Vec<_>>();

    paths.sort();
    paths
}

fn other_color(color: u8) -> u8 {
    if color == BLACK { WHITE } else { BLACK }
}

fn on_board(row: isize, col: isize) -> bool {
    row >= 0 && row < BOARD_SIZE as isize && col >= 0 && col < BOARD_SIZE as isize
}

fn group_has_liberty(
    board: &Board,
    start_row: usize,
    start_col: usize,
    visited: &mut Board,
) -> bool {
    let color = board[start_row][start_col];
    let mut stack = [(0usize, 0usize); BOARD_SIZE * BOARD_SIZE];
    let mut stack_size = 0usize;

    stack[stack_size] = (start_row, start_col);
    stack_size += 1;
    visited[start_row][start_col] = 1;

    while stack_size > 0 {
        stack_size -= 1;
        let (row, col) = stack[stack_size];

        for (row_delta, col_delta) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let next_row = row as isize + row_delta;
            let next_col = col as isize + col_delta;

            if !on_board(next_row, next_col) {
                continue;
            }

            let next_row = next_row as usize;
            let next_col = next_col as usize;

            if board[next_row][next_col] == EMPTY {
                return true;
            }

            if board[next_row][next_col] == color && visited[next_row][next_col] == 0 {
                visited[next_row][next_col] = 1;
                stack[stack_size] = (next_row, next_col);
                stack_size += 1;
            }
        }
    }

    false
}

fn remove_group(board: &mut Board, start_row: usize, start_col: usize) {
    let color = board[start_row][start_col];
    let mut stack = [(0usize, 0usize); BOARD_SIZE * BOARD_SIZE];
    let mut stack_size = 0usize;

    stack[stack_size] = (start_row, start_col);
    stack_size += 1;
    board[start_row][start_col] = EMPTY;

    while stack_size > 0 {
        stack_size -= 1;
        let (row, col) = stack[stack_size];

        for (row_delta, col_delta) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let next_row = row as isize + row_delta;
            let next_col = col as isize + col_delta;

            if !on_board(next_row, next_col) {
                continue;
            }

            let next_row = next_row as usize;
            let next_col = next_col as usize;

            if board[next_row][next_col] == color {
                board[next_row][next_col] = EMPTY;
                stack[stack_size] = (next_row, next_col);
                stack_size += 1;
            }
        }
    }
}

fn play_move(board: &mut Board, encoded_move: u16, color: u8) {
    if encoded_move == PASS_MOVE {
        return;
    }

    let col = encoded_move as usize / BOARD_SIZE;
    let row = encoded_move as usize % BOARD_SIZE;
    assert!(row < BOARD_SIZE && col < BOARD_SIZE, "move out of bounds");
    assert_eq!(board[row][col], EMPTY, "move on occupied point");

    board[row][col] = color;

    for (row_delta, col_delta) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let next_row = row as isize + row_delta;
        let next_col = col as isize + col_delta;

        if !on_board(next_row, next_col) {
            continue;
        }

        let next_row = next_row as usize;
        let next_col = next_col as usize;

        if board[next_row][next_col] == other_color(color) {
            let mut visited = [[0u8; BOARD_SIZE]; BOARD_SIZE];

            if !group_has_liberty(board, next_row, next_col, &mut visited) {
                remove_group(board, next_row, next_col);
            }
        }
    }

    let mut visited = [[0u8; BOARD_SIZE]; BOARD_SIZE];
    assert!(
        group_has_liberty(board, row, col, &mut visited),
        "suicide move"
    );
}

fn board_snapshot(board: &Board) -> String {
    let mut snapshot = String::with_capacity(BOARD_SIZE * BOARD_SIZE);

    for row in board {
        for cell in row {
            snapshot.push(char::from(b'0' + *cell));
        }
    }

    snapshot
}

fn expected_snapshots(root: &Path) -> Vec<Vec<String>> {
    fixture_paths(root)
        .iter()
        .map(|path| {
            let sgf = fs::read_to_string(path).expect("failed to read SGF fixture");
            let moves = parse_sgf_moves(&sgf, BOARD_SIZE as u32);
            let mut board = [[0u8; BOARD_SIZE]; BOARD_SIZE];
            let mut snapshots = Vec::new();

            for (move_index, encoded_move) in moves.iter().enumerate() {
                let color = if move_index % 2 == 0 { BLACK } else { WHITE };
                play_move(&mut board, *encoded_move, color);
                snapshots.push(board_snapshot(&board));
            }

            snapshots
        })
        .collect()
}

fn validator_snapshots(root: &Path) -> Vec<Vec<String>> {
    let generated_dir = root.join("../test_generated");
    fs::create_dir_all(&generated_dir).expect("failed to create generated test directory");

    let games = compress_games_from_directory("tests/fixtures/sgf", BOARD_SIZE as u32, 10)
        .expect("failed to compress SGF fixtures");
    generate_c_header(&games, generated_dir.join("games_data.h").to_str().unwrap())
        .expect("failed to generate test games_data.h");

    let binary = root.join("target/validate_games");
    let compile_status = Command::new("c++")
        .current_dir(root)
        .args([
            "-std=c++11",
            "-I",
            "../logic",
            "-I",
            "../test_generated",
            "tests/validator/validate_games.cpp",
            "../logic/baduk_engine.cpp",
            "-o",
        ])
        .arg(&binary)
        .status()
        .expect("failed to compile C++ validator");

    assert!(compile_status.success(), "C++ validator compilation failed");

    let output = Command::new(&binary)
        .current_dir(root)
        .output()
        .expect("failed to run C++ validator");

    assert!(
        output.status.success(),
        "C++ validator failed:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let mut games = Vec::new();
    let mut current_game = Vec::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.starts_with("game ") {
            if !current_game.is_empty() {
                games.push(current_game);
                current_game = Vec::new();
            }
        } else if let Some(snapshot) = line
            .strip_prefix("move ")
            .and_then(|rest| rest.split_once(' ').map(|(_, snapshot)| snapshot))
        {
            current_game.push(snapshot.to_string());
        }
    }

    if !current_game.is_empty() {
        games.push(current_game);
    }

    games
}

#[test]
fn generated_headers_match_sgf_replay() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(validator_snapshots(root), expected_snapshots(root));
}
