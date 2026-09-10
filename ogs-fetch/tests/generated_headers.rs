use goban::pieces::stones::Color;
use goban::rules::game::Game;
use goban::rules::{GobanSizes, Move, CHINESE};
use ogs_fetch::compression::{compress_games_from_directory, generate_c_header, parse_sgf_moves};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BOARD_SIZE: usize = 19;
const PASS_MOVE: u16 = 0xFFFF;

fn fixture_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(root.join("tests/fixtures/sgf"))
        .expect("missing SGF fixtures")
        .map(|entry| entry.expect("failed to read fixture entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sgf"))
        .collect::<Vec<_>>();

    paths.sort();
    paths
}

fn board_snapshot(game: &Game) -> String {
    let mut snapshot = String::with_capacity(BOARD_SIZE * BOARD_SIZE);

    for row in game.goban().matrix() {
        for cell in row {
            let value = match cell {
                None => '0',
                Some(Color::Black) => '1',
                Some(Color::White) => '2',
            };
            snapshot.push(char::from(value));
        }
    }

    snapshot
}

fn goban_snapshots(root: &Path) -> Vec<Vec<String>> {
    fixture_paths(root)
        .iter()
        .map(|path| {
            let sgf = fs::read_to_string(path).expect("failed to read SGF fixture");
            let moves = parse_sgf_moves(&sgf, BOARD_SIZE as u32);
            let mut game = Game::new(GobanSizes::Nineteen, CHINESE);
            let mut snapshots = Vec::new();

            for (move_index, encoded_move) in moves.iter().enumerate() {
                let move_to_play = if *encoded_move == PASS_MOVE {
                    Move::Pass
                } else {
                    let col = *encoded_move / BOARD_SIZE as u16;
                    let row = *encoded_move % BOARD_SIZE as u16;
                    Move::Play(row as u8, col as u8)
                };

                game.try_play(move_to_play).unwrap_or_else(|error| {
                    panic!(
                        "Goban rejected move {} in {:?}: {error:?}",
                        move_index + 1,
                        path
                    )
                });
                snapshots.push(board_snapshot(&game));
            }

            snapshots
        })
        .collect()
}

fn validator_snapshots(root: &Path) -> Vec<Vec<String>> {
    let generated_dir = root.join("../test-artifacts");
    if generated_dir.exists() {
        fs::remove_dir_all(&generated_dir).expect("failed to clean test artifacts");
    }
    fs::create_dir_all(&generated_dir).expect("failed to create generated test directory");

    let games = compress_games_from_directory("tests/fixtures/sgf", BOARD_SIZE as u32, 10)
        .expect("failed to compress SGF fixtures");
    generate_c_header(
        &games,
        generated_dir
            .join("generated_games_data.h")
            .to_str()
            .unwrap(),
    )
    .expect("failed to generate test generated_games_data.h");

    let binary = root.join("target/validate_games");
    let compile_status = Command::new("c++")
        .current_dir(root)
        .args([
            "-std=c++11",
            "-I",
            "../engine",
            "-I",
            "../test-artifacts",
            "tests/validator/validate_games.cpp",
            "../engine/baduk_engine.cpp",
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
    assert_eq!(validator_snapshots(root), goban_snapshots(root));
}
