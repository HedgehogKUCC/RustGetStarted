use macroquad::prelude::*;

const TILE_SIZE: f32 = 48.0;
const ROWS: usize = 11;
const COLS: usize = 13;
const BOMB_RANGE: i32 = 2;

#[derive(Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    Brick,
}

struct Player {
    x: usize,
    y: usize,
}

struct Bomb {
    x: usize,
    y: usize,
    timer: f32,
}

struct Explosion {
    x: usize,
    y: usize,
    timer: f32,
}

#[macroquad::main("Bomber Game")]
async fn main() {
    let mut map = create_map();
    let mut player = Player { x: 1, y: 1 };
    let mut bombs: Vec<Bomb> = Vec::new();
    let mut explosions: Vec<Explosion> = Vec::new();
    let mut game_over = false;

    loop {
        let delta_time = get_frame_time();

        clear_background(BLACK);

        if !game_over {
            handle_player_input(&map, &mut player);
            handle_bomb_input(&player, &mut bombs);

            update_bombs(&mut map, &mut bombs, &mut explosions, delta_time);

            if player_hit_by_explosion(&player, &explosions) {
                game_over = true;
            }

            update_explosions(&mut explosions, delta_time);
        }

        draw_map(&map);
        draw_bombs(&bombs);
        draw_explosions(&explosions);
        draw_player(&player);

        if game_over {
            draw_game_over();
        }

        next_frame().await;
    }
}

fn update_bombs(
    map: &mut [[Tile; COLS]; ROWS],
    bombs: &mut Vec<Bomb>,
    explosions: &mut Vec<Explosion>,
    delta_time: f32
) {
    for bomb in bombs.iter_mut() {
        bomb.timer -= delta_time;
    }

    let mut new_explosions = Vec::new();

    bombs.retain(|bomb| {
        if bomb.timer <= 0.0 {
            new_explosions.extend(create_explosion_area(map, bomb.x, bomb.y));

            false
        } else {
            true
        }
    });

    explosions.extend(new_explosions);
}

fn update_explosions(explosions: &mut Vec<Explosion>, delta_time: f32) {
    for explosion in explosions.iter_mut() {
        explosion.timer -= delta_time;
    }

    explosions.retain(|explosion| explosion.timer > 0.0);
}

fn handle_bomb_input(player: &Player, bombs: &mut Vec<Bomb>) {
    if is_key_pressed(KeyCode::Space) {
        let already_has_bomb = bombs
            .iter()
            .any(|bomb| bomb.x == player.x && bomb.y == player.y);

        if !already_has_bomb {
            bombs.push(Bomb {
                x: player.x,
                y: player.y,
                timer: 2.0,
            });
        }
    }
}

fn handle_player_input(map: &[[Tile; COLS]; ROWS], player: &mut Player) {
    if is_key_pressed(KeyCode::Up) {
        try_move_player(map, player, 0 , -1);
    } else if is_key_pressed(KeyCode::Down) {
        try_move_player(map, player, 0, 1);
    } else if is_key_pressed(KeyCode::Left) {
        try_move_player(map, player, -1, 0);
    } else if is_key_pressed(KeyCode::Right) {
        try_move_player(map, player, 1, 0);
    }
}

fn try_move_player(map: &[[Tile; COLS]; ROWS], player: &mut Player, dx: i32, dy: i32) {
    let next_x = player.x as i32 + dx;
    let next_y = player.y as i32 + dy;

    if is_walkable(map, next_x, next_y) {
        player.x = next_x as usize;
        player.y = next_y as usize;
    }
}

fn is_walkable(map: &[[Tile; COLS]; ROWS], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= COLS as i32 || y >= ROWS as i32 {
        return false;
    }

    matches!(map[y as usize][x as usize], Tile::Empty)
}

fn create_map() -> [[Tile; COLS]; ROWS] {
    let mut map = [[Tile::Empty; COLS]; ROWS];

    for y in 0..ROWS {
        for x in 0..COLS {
            if y == 0 || y == ROWS - 1 || x == 0 || x == COLS - 1 {
                map[y][x] = Tile::Wall;
            } else if x % 2 == 0 && y % 2 == 0 {
                map[y][x] = Tile::Wall;
            } else if (x+y) % 3 == 0 {
                map[y][x] = Tile::Brick;
            }
        }
    }

    // 確保玩家出生點附近是空地，之後移動時比較好測
    map[1][1] = Tile::Empty;
    map[1][2] = Tile::Empty;
    map[2][1] = Tile::Empty;

    map
}

fn draw_map(map: &[[Tile; COLS]; ROWS]) {
    for y in 0..ROWS {
        for x in 0..COLS {
            let color = match map[y][x] {
                Tile::Empty => DARKGRAY,
                Tile::Wall => GRAY,
                Tile::Brick => BROWN,
            };

            draw_rectangle(
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                TILE_SIZE - 2.0,
                TILE_SIZE - 2.0,
                color,
            );
        }
    }
}

fn draw_player(player: &Player) {
    draw_rectangle(
        player.x as f32 * TILE_SIZE + 8.0,
        player.y as f32 * TILE_SIZE + 8.0,
        TILE_SIZE - 16.0,
        TILE_SIZE - 16.0,
        BLUE,
    );
}

fn draw_bombs(bombs: &[Bomb]) {
    for bomb in bombs {
        draw_circle(
            bomb.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            bomb.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            TILE_SIZE * 0.3,
            BLACK,
        )
    }
}

fn draw_explosions(explosions: &[Explosion]) {
    for explosion in explosions {
        draw_rectangle(
            explosion.x as f32 * TILE_SIZE + 4.0,
            explosion.y as f32 * TILE_SIZE + 4.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            ORANGE
        )
    }
}

fn create_explosion_area(
    map: &mut [[Tile; COLS]; ROWS],
    bomb_x: usize,
    bomb_y: usize,
) -> Vec<Explosion> {
    let mut explosions = Vec::new();

    explosions.push(Explosion {
        x: bomb_x,
        y: bomb_y,
        timer: 0.4,
    });

    let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for (dx, dy) in directions {
        for distance in 1..=BOMB_RANGE {
            let x = bomb_x as i32 + dx * distance;
            let y = bomb_y as i32 + dy * distance;

            if x < 0 || y < 0 || x >= COLS as i32 || y >= ROWS as i32 {
                break;
            }
            
            let tile_x = x as usize;
            let tile_y = y as usize;

            match map[tile_y][tile_x] {
                Tile::Wall => break,
                Tile::Brick => {
                    explosions.push(Explosion {
                        x: tile_x,
                        y: tile_y,
                        timer: 0.4,
                    });

                    map[tile_y][tile_x] = Tile::Empty;
                    break;
                },
                Tile::Empty => explosions.push(Explosion {
                    x: tile_x,
                    y: tile_y,
                    timer: 0.4,
                }),
            }
        }
    }

    explosions
}

fn player_hit_by_explosion(player: &Player, explosions: &[Explosion]) -> bool {
    explosions
        .iter()
        .any(|explosion| explosion.x == player.x && explosion.y == player.y)
}

fn draw_game_over() {
    let text = "GAME OVER";
    let font_size = 60.0;
    let text_size = measure_text(text, None, font_size as u16, 1.0);

    draw_text(
        text,
        screen_width() / 2.0 - text_size.width / 2.0,
        screen_height() / 2.0,
        font_size,
        RED,
    );
}
