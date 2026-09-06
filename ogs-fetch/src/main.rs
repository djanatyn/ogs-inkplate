use clap::Subcommand;
use ogs_fetch::{ApiResponse, Database, Game, compression, is_valid_game_outcome};
use std::thread;
use std::time::Duration;

#[derive(clap::Parser)]
#[command(name = "ogs-fetch")]
#[command(about = "Fetch and compress Go games from Online-Go.com", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch game results from OGS API
    FetchResults {
        /// User ID to fetch games for
        #[arg(long)]
        user_id: u64,

        /// Page size (max 100)
        #[arg(long, default_value = "100")]
        page_size: u32,

        /// Board size (9 or 19)
        #[arg(long, default_value = "19")]
        board_size: u32,

        /// Database path
        #[arg(long, default_value = "games.db")]
        db: String,
    },

    /// Download SGF files for games
    FetchGames {
        /// Output directory for SGF files
        #[arg(long, default_value = "sgf")]
        output_dir: String,

        /// Board size (9 or 19)
        #[arg(long, default_value = "19")]
        board_size: u32,

        /// Database path
        #[arg(long, default_value = "games.db")]
        db: String,
    },

    /// Compress downloaded games
    CompressGames {
        /// Input directory with SGF files
        #[arg(long, default_value = "sgf")]
        input_dir: String,

        /// Board size (9 or 19)
        #[arg(long, default_value = "19")]
        board_size: u32,

        /// Output directory for C header files
        #[arg(long, default_value = ".")]
        output_dir: String,

        /// Maximum number of games to compress
        #[arg(long, default_value = "50")]
        max_games: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = <Cli as clap::Parser>::parse();

    match cli.command {
        Commands::FetchResults {
            user_id,
            page_size,
            board_size,
            db,
        } => {
            fetch_results(user_id, page_size, board_size, &db)?;
        }
        Commands::FetchGames {
            output_dir,
            board_size,
            db,
        } => {
            fetch_games(&output_dir, board_size, &db)?;
        }
        Commands::CompressGames {
            input_dir,
            board_size,
            output_dir,
            max_games,
        } => {
            compress_games(&input_dir, board_size, &output_dir, max_games)?;
        }
    }

    Ok(())
}

fn fetch_results(
    user_id: u64,
    page_size: u32,
    board_size: u32,
    db_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Fetching game results for user {} (board size: {}x{}, page size: {})",
        user_id, board_size, board_size, page_size
    );

    let db = Database::new(db_path)?;
    db.init_schema()?;

    let mut page = db.get_last_page()?;
    println!("Resuming from page {}", page);

    loop {
        page += 1;
        println!("Fetching page {}...", page);

        let url = format!(
            "https://online-go.com/api/v1/players/{}/games/?page_size={}&page={}&source=play&ended__isnull=false&height={}&width={}&ordering=-ended",
            user_id, page_size, page, board_size, board_size
        );

        let response = ureq::get(&url).call();
        let mut response = match response {
            Ok(resp) => resp,
            Err(ureq::Error::StatusCode(404)) => {
                println!("Reached end of results (page {} not found)", page);
                break;
            }
            Err(e) => return Err(Box::new(e)),
        };
        let body = response.body_mut().read_to_string()?;
        let api_response: ApiResponse = serde_json::from_str(&body)?;

        println!("  Found {} games on this page", api_response.results.len());

        let mut valid_count = 0;
        for result in api_response.results {
            // Skip games with handicap (only want even games)
            if result.handicap > 0 {
                println!(
                    "    Skipping handicap game {} (handicap: {})",
                    result.id, result.handicap
                );
                continue;
            }

            // Skip games that didn't complete normally
            if !is_valid_game_outcome(&result.outcome) {
                println!(
                    "    Skipping game {} (outcome: {})",
                    result.id, result.outcome
                );
                continue;
            }

            let game = Game {
                game_id: result.id,
                black_player: result.players.black.username.clone(),
                white_player: result.players.white.username.clone(),
                result: result.outcome.clone(),
                date: result.ended.clone(),
                downloaded: false,
            };
            db.insert_game(&game)?;
            valid_count += 1;
        }

        println!("  Added {} valid games to database", valid_count);

        // Update last page
        db.set_last_page(page)?;

        // Rate limiting: 1 request per second
        thread::sleep(Duration::from_secs(1));

        // Check if there are more pages
        if api_response.next.is_none() {
            println!("Reached end of results");
            break;
        }
    }

    let total_games = db.count_games()?;
    println!("Total games in database: {}", total_games);

    Ok(())
}

fn fetch_games(
    output_dir: &str,
    board_size: u32,
    db_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Fetching SGF files to directory: {} (board size: {}x{})",
        output_dir, board_size, board_size
    );

    std::fs::create_dir_all(output_dir)?;

    let db = Database::new(db_path)?;

    let undownloaded = db.get_undownloaded_games()?;
    let undownloaded_count = undownloaded.len();
    println!("Found {} games to download", undownloaded_count);

    let total = db.count_games()?;
    let mut count = 0;

    for game_id in &undownloaded {
        let sgf_path = format!("{}/{}.sgf", output_dir, game_id);

        // Skip if already downloaded
        if std::path::Path::new(&sgf_path).exists() {
            println!("  Game {} already exists, skipping", game_id);
            db.mark_game_downloaded(*game_id)?;
            continue;
        }

        let url = format!("https://online-go.com/api/v1/games/{}/sgf", game_id);

        match ureq::get(&url).call() {
            Ok(mut response) => {
                let sgf_data = response.body_mut().read_to_string()?;
                std::fs::write(&sgf_path, sgf_data)?;
                db.mark_game_downloaded(*game_id)?;
                count += 1;

                let downloaded = db.count_downloaded_games()?;
                println!(
                    "Downloaded {}/{} ({} total) - Game {}",
                    count, undownloaded_count, downloaded, game_id
                );

                // Rate limiting
                thread::sleep(Duration::from_millis(1000));
            }
            Err(e) => {
                eprintln!("Failed to download game {}: {}", game_id, e);
            }
        }
    }

    println!("Download complete!");
    let downloaded = db.count_downloaded_games()?;
    println!("Downloaded {} out of {} total games", downloaded, total);

    Ok(())
}

fn compress_games(
    input_dir: &str,
    board_size: u32,
    output_dir: &str,
    max_games: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Compressing games from directory: {} (board size: {}x{})",
        input_dir, board_size, board_size
    );
    println!("Max games to compress: {}", max_games);

    // Compress games from SGF files
    let games = compression::compress_games_from_directory(input_dir, board_size, max_games)?;

    if games.is_empty() {
        println!("No games found to compress!");
        return Ok(());
    }

    // Calculate statistics
    let total_moves: usize = games.iter().map(|g| g.moves.len()).sum();
    let avg_moves = total_moves / games.len();

    println!("\nCompression Statistics:");
    println!("  Total games: {}", games.len());
    println!("  Total moves: {}", total_moves);
    println!("  Average moves per game: {}", avg_moves);

    // Estimate size
    let estimated_size = 2 + (games.len() * 2) + total_moves * 2 + games.len();
    println!("  Estimated data size: {} bytes", estimated_size);

    // Generate C header files in output directory
    let games_data_path = format!("{}/games_data.h", output_dir);
    let games_metadata_path = format!("{}/games_metadata.h", output_dir);

    println!("\nGenerating C header files in: {}", output_dir);
    compression::generate_c_header(&games, &games_data_path)?;
    println!("  Generated: {}", games_data_path);

    compression::generate_metadata_header(&games, &games_metadata_path)?;
    println!("  Generated: {}", games_metadata_path);

    println!("Compression complete!");

    Ok(())
}
