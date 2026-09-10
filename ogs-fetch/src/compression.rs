use sgf_parse::{go::Prop, parse};
use std::fs;

pub const BADUK_PASS_MOVE: u16 = 0xFFFF;

/// Parse SGF moves from a game file
/// Extracts moves in the format ;B[pd] or ;W[cd]
/// Only parses the main line, stops at variation branches
pub fn parse_sgf_moves(content: &str, board_size: u32) -> Vec<u16> {
    let mut moves = Vec::new();

    // Parse SGF with sgf-parse library
    // parse() returns Vec<GameTree>
    let Ok(game_trees) = parse(content) else {
        return moves;
    };

    // Get the first game tree
    let Some(game_tree) = game_trees.first() else {
        return moves;
    };

    // Convert GameTree to SgfNode and get the root node
    let Ok(root_node) = game_tree.as_go_node() else {
        return moves;
    };

    // Iterate through the main variation
    for node in root_node.main_variation() {
        // Check for move properties (B for black, W for white)
        for prop in node.properties() {
            match prop {
                Prop::B(mv) | Prop::W(mv) => {
                    // Convert move to our coordinate format and encode
                    if let Some(encoded) = encode_move_from_lib(mv, board_size) {
                        moves.push(encoded);
                    }
                }
                _ => {}
            }
        }
    }

    moves
}

/// convert sgf-parse Move to u16 encoding
fn encode_move_from_lib(mv: &sgf_parse::go::Move, board_size: u32) -> Option<u16> {
    match mv {
        sgf_parse::go::Move::Pass => Some(BADUK_PASS_MOVE),
        sgf_parse::go::Move::Move(point) => {
            // Convert Point (x, y) to SGF coordinate string
            // Point.x becomes the first character, Point.y becomes the second character
            let coords = format!("{}{}", (point.x + b'a') as char, (point.y + b'a') as char);
            encode_move(&coords, board_size)
        }
    }
}

/// encode SGF coordinates (e.g., "pd") to a single number
/// SGF uses letters a-s (0-18) for 19x19, a-i (0-8) for 9x9
/// position = col * board_size + row (col is first char, row is second char)
fn encode_move(coords: &str, board_size: u32) -> Option<u16> {
    if coords.len() < 2 {
        return None;
    }

    let col = coords.chars().next()? as u32;
    let row = coords.chars().nth(1)? as u32;

    // 'a'=0, 'b'=1, 's'=18
    let col_num = col.saturating_sub('a' as u32);
    let row_num = row.saturating_sub('a' as u32);

    // check if coordinates are valid for the given board size
    if col_num >= board_size || row_num >= board_size {
        return None;
    }

    let position = (col_num * board_size + row_num) as u16;
    Some(position)
}

/// Extract SGF property value from content
/// Looks for pattern like "DT[2017-04-10]" and returns "2017-04-10"
fn extract_sgf_property(content: &str, property: &str) -> String {
    let pattern = format!("{}[", property);
    if let Some(start) = content.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = content[start..].find(']') {
            return content[start..start + end].to_string();
        }
    }
    String::new()
}

/// Check if a game has handicap stones (HA property)
fn has_handicap(content: &str) -> bool {
    // Look for HA[n] property where n > 0
    if let Some(start) = content.find("HA[") {
        let start = start + 3;
        if let Some(end) = content[start..].find(']')
            && let Ok(handicap_str) = content[start..start + end].parse::<i32>()
        {
            return handicap_str > 0;
        }
    }
    false
}

/// Extract game metadata from SGF content
fn extract_metadata(content: &str) -> GameMetadata {
    GameMetadata {
        date: extract_sgf_property(content, "DT"),
        black_player: extract_sgf_property(content, "PB"),
        white_player: extract_sgf_property(content, "PW"),
        black_rank: extract_sgf_property(content, "BR"),
        white_rank: extract_sgf_property(content, "WR"),
    }
}

/// Game metadata structure
pub struct GameMetadata {
    pub date: String, // YYYY-MM-DD format
    pub black_player: String,
    pub white_player: String,
    pub black_rank: String,
    pub white_rank: String,
}

/// Compressed game data structure
pub struct CompressedGame {
    pub moves: Vec<u16>,
    pub filename: String,
    pub metadata: GameMetadata,
}

/// Compress all SGF files in a directory
pub fn compress_games_from_directory(
    input_dir: &str,
    board_size: u32,
    max_games: usize,
) -> Result<Vec<CompressedGame>, Box<dyn std::error::Error>> {
    let mut games = Vec::new();
    let mut file_count = 0;

    let mut entries = fs::read_dir(input_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    // Read all .sgf files from directory
    for entry in entries {
        if file_count >= max_games {
            break;
        }

        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sgf")
            && let Ok(content) = fs::read_to_string(&path)
        {
            // Skip games with handicap stones
            if has_handicap(&content) {
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown.sgf")
                    .to_string();
                println!("  Skipping handicap game: {}", filename);
                continue;
            }

            let moves = parse_sgf_moves(&content, board_size);
            if !moves.is_empty() {
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown.sgf")
                    .to_string();
                let metadata = extract_metadata(&content);
                games.push(CompressedGame {
                    moves,
                    filename,
                    metadata,
                });
                file_count += 1;
                let game_ref = games.last().unwrap();
                println!(
                    "  Compressed game {} ({} moves) from {} ({} vs {} on {})",
                    file_count,
                    game_ref.moves.len(),
                    game_ref.filename,
                    game_ref.metadata.black_player,
                    game_ref.metadata.white_player,
                    game_ref.metadata.date
                );
            }
        }
    }

    println!("Compressed {} games total", games.len());
    Ok(games)
}

/// Escape a string for use in C string literal
fn escape_c_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Generate a C header file with game metadata
pub fn generate_metadata_header(
    games: &[CompressedGame],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut header = String::new();
    header.push_str("#ifndef GENERATED_GAMES_METADATA_H\n");
    header.push_str("#define GENERATED_GAMES_METADATA_H\n\n");
    header.push_str("#include \"game_metadata.h\"\n");
    header.push_str("#include \"../engine/baduk_platform.h\"\n\n");

    // Generate string constants in PROGMEM
    header.push_str("// Metadata strings (stored in PROGMEM)\n");
    for (game_idx, game) in games.iter().enumerate() {
        header.push_str(&format!(
            "const char GAME_{}_DATE[] PROGMEM = \"{}\";\n",
            game_idx,
            escape_c_string(&game.metadata.date)
        ));
        header.push_str(&format!(
            "const char GAME_{}_BLACK[] PROGMEM = \"{}\";\n",
            game_idx,
            escape_c_string(&game.metadata.black_player)
        ));
        header.push_str(&format!(
            "const char GAME_{}_WHITE[] PROGMEM = \"{}\";\n",
            game_idx,
            escape_c_string(&game.metadata.white_player)
        ));
        header.push_str(&format!(
            "const char GAME_{}_BLACK_RANK[] PROGMEM = \"{}\";\n",
            game_idx,
            escape_c_string(&game.metadata.black_rank)
        ));
        header.push_str(&format!(
            "const char GAME_{}_WHITE_RANK[] PROGMEM = \"{}\";\n",
            game_idx,
            escape_c_string(&game.metadata.white_rank)
        ));
    }

    header.push_str("\n// Metadata array\n");
    header.push_str("const GameMetadata GAMES_METADATA[] PROGMEM = {\n");
    for game_idx in 0..games.len() {
        header.push_str(&format!(
            "    {{ GAME_{}_DATE, GAME_{}_BLACK, GAME_{}_WHITE, GAME_{}_BLACK_RANK, GAME_{}_WHITE_RANK }},\n",
            game_idx, game_idx, game_idx, game_idx, game_idx
        ));
    }
    header.push_str("};\n\n");
    header.push_str("#endif\n");

    fs::write(output_path, header)?;
    Ok(())
}

/// Generate a C header file with compressed game data
pub fn generate_c_header(
    games: &[CompressedGame],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut header = String::new();
    header.push_str("#ifndef GENERATED_GAMES_DATA_H\n");
    header.push_str("#define GENERATED_GAMES_DATA_H\n\n");
    header.push_str("#include \"../engine/baduk_platform.h\"\n");
    header.push_str("#include \"../engine/baduk_types.h\"\n\n");

    for (game_idx, game) in games.iter().enumerate() {
        header.push_str(&format!(
            "// Game {}: {} moves (from {})\n",
            game_idx,
            game.moves.len(),
            game.filename
        ));
        header.push_str(&format!(
            "const uint16_t GAME_{}_MOVES[] PROGMEM = {{\n",
            game_idx
        ));

        // Write moves in groups of 8 for readability.
        for (move_idx, mv) in game.moves.iter().enumerate() {
            if move_idx % 8 == 0 {
                header.push_str("    ");
            }

            if *mv == BADUK_PASS_MOVE {
                header.push_str("BADUK_PASS_MOVE");
            } else {
                header.push_str(&mv.to_string());
            }

            let is_last_move = move_idx == game.moves.len() - 1;
            header.push(',');

            if !is_last_move {
                header.push(' ');
            }

            if (move_idx + 1) % 8 == 0 || is_last_move {
                header.push('\n');
            }
        }

        header.push_str("};\n\n");
    }

    header.push_str("const BadukGameRecord GAMES[] PROGMEM = {\n");
    for (game_idx, game) in games.iter().enumerate() {
        header.push_str(&format!(
            "    {{ GAME_{}_MOVES, {} }},\n",
            game_idx,
            game.moves.len()
        ));
    }
    header.push_str("};\n\n");
    header.push_str(&format!("const uint16_t GAME_COUNT = {};\n\n", games.len()));
    header.push_str("#endif\n");

    fs::write(output_path, header)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_move() {
        assert_eq!(encode_move("aa", 19), Some(0));
        assert_eq!(encode_move("pd", 19), Some(15 * 19 + 3));
        assert_eq!(encode_move("ss", 19), Some(18 * 19 + 18));
    }

    #[test]
    fn test_parse_sgf_moves() {
        // Test with proper SGF game structure
        let sgf = "(;FF[4]GM[1];B[pd];W[dp];B[pq])";
        let moves = parse_sgf_moves(sgf, 19);
        assert_eq!(moves.len(), 3);

        // Verify first move is pd -> (15, 3) -> 15*19+3 = 288
        assert_eq!(moves[0], 288);
    }

    #[test]
    fn test_parse_sgf_moves_stops_at_variations() {
        // Test that parser follows main line (first child at each node)
        // The SGF (;B[pd](;W[dp])(;W[pp])) has B[pd] with two child variations
        // main_variation() includes the first child, so we get both B[pd] and W[dp]
        let sgf = "(;FF[4]GM[1];B[pd](;W[dp])(;W[pp]))";
        let moves = parse_sgf_moves(sgf, 19);
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0], 288); // pd
        // moves[1] is W[dp]: "dp" = (3, 15) = 3 * 19 + 15 = 72
        assert_eq!(moves[1], 72);
    }

    #[test]
    fn test_handicap_detection() {
        // Game with handicap stones
        let sgf_with_handicap = "(;FF[4]GM[1]HA[2];B[pd];W[dp])";
        assert!(has_handicap(sgf_with_handicap));

        // Game without handicap stones
        let sgf_no_handicap = "(;FF[4]GM[1];B[pd];W[dp])";
        assert!(!has_handicap(sgf_no_handicap));

        // Game with HA[0] should not be treated as handicap
        let sgf_zero_handicap = "(;FF[4]GM[1]HA[0];B[pd];W[dp])";
        assert!(!has_handicap(sgf_zero_handicap));
    }
}
