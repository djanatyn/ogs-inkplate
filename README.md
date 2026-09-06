ogs-inkplate
============

display online-go.com games on an inkplate TEMPERA4

<p align="center">
  <img src="https://raw.githubusercontent.com/djanatyn/ogs-inkplate/main/screenshot.png" alt="screenshot of baduk game displayed on tempera4"></img>
  <img src="https://raw.githubusercontent.com/djanatyn/ogs-inkplate/main/photo.webp" alt="baduk game displayed on tempera4"></img>
</p>

## fetching game results

given a user ID, the `ogs-fetch` crate has a `fetch-resutls` subcommand that can download game results from the OGS API:

``` sh
$ cargo run --bin ogs-fetch -- fetch-results \
    --user-id 435842 \
    --board-size 9 \
    --db ./9x9.db
```
```
Fetching game results for user 435842 (board size: 9x9, page size: 100)
Resuming from page 0
Fetching page 1...
  Found 100 games on this page
    Skipping game 90135308 (outcome: Resignation)
    Skipping handicap game 88473662 (handicap: 1)
    Skipping game 82379706 (outcome: Timeout)
...
    Skipping handicap game 58273843 (handicap: 1)
  Added 32 valid games to database
Reached end of results
Total games in database: 413
```

game results are filtered before persisting to the database:
```rust
// Skip games with handicap (only want even games)
if result.handicap > 0 {
    println!(
        "    Skipping handicap game {} (handicap: {})",
        result.id, result.handicap
    );
    continue;
}
```
```rust
pub fn is_valid_game_outcome(outcome: &str) -> bool {
    // Only include games that completed normally (by points)
    // Skip: Resignation, Timeout, Disconnection, Cancelled, etc.
    !outcome.contains("Resignation")
        && !outcome.contains("Timeout")
        && !outcome.contains("Disconnect")
        && !outcome.contains("Cancelled")
        && !outcome.contains("Annulled")
        // Valid outcomes are point-based scores like "6.5 points", "W+24.5", etc.
        && (outcome.contains("points") || outcome.contains("+"))
}
```

we can inspect our game database:
```sql
$ sqlite3 -line 9x9.db \
    'select * from games order by game_id desc limit 3;'
```
```
     game_id = 88459729
black_player = rockeazylee
white_player = djanatyn
      result = 10.5 points
        date = 2026-07-04T04:06:39.027118Z
  downloaded = 0
  created_at = 2026-09-06 14:37:53

     game_id = 87862913
black_player = djanatyn
white_player = 6.5whitewins
      result = 10.5 points
        date = 2026-06-13T12:25:43.129531Z
  downloaded = 0
  created_at = 2026-09-06 14:37:53

     game_id = 85878254
black_player = Termessos
white_player = djanatyn
      result = 0.5 points
        date = 2026-04-08T17:32:03.671800Z
  downloaded = 0
  created_at = 2026-09-06 14:37:53
```

## fetch SGF files

SGF files contain the moves of each game. the download runs slowly to reduce load for OGS infrastructure:
```sh
; cargo run --bin ogs-fetch -- fetch-games \
  --board-size 9 \
  --db ./9x9.db \
  --output-dir ./9x9-sgfs
```
```
Fetching SGF files to directory: ./9x9-sgfs (board size: 9x9)
Found 413 games to download
Downloaded 1/413 (1 total) - Game 8321848
Downloaded 2/413 (2 total) - Game 8695206
Downloaded 3/413 (3 total) - Game 8878374
Downloaded 4/413 (4 total) - Game 8896227
Downloaded 5/413 (5 total) - Game 9032030
...
```
