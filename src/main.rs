use macroquad::prelude::*;

const TILE_SIZE: f32 = 48.0;
const PLAYER_MOVE_DELAY: f32 = 0.15;
const ROWS: usize = 11;
const COLS: usize = 13;
// const BOMB_RANGE: i32 = 2;

#[derive(Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    Brick,
}

struct Player {
    x: usize,
    y: usize,
    bomb_range: i32,
    max_bombs: usize,
    move_timer: f32,
}

struct Bomb {
    x: usize,
    y: usize,
    timer: f32,
    range: i32,
}

struct Explosion {
    x: usize,
    y: usize,
    timer: f32,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Enemy {
    x: usize,
    y: usize,
    direction: Direction,
    move_timer: f32,
}

#[derive(Clone, Copy)]
enum PowerUpKind {
    Range,
    Bomb,
}

struct PowerUp {
    x: usize,
    y: usize,
    kind: PowerUpKind,
}

#[macroquad::main("Bomber Game")]
async fn main() {
    let mut map = create_map();
    let player_texture = load_texture("assets/player.png").await.unwrap();
    player_texture.set_filter(FilterMode::Nearest);
    let bomb_texture = load_texture("assets/bomb.png").await.unwrap();
    bomb_texture.set_filter(FilterMode::Nearest);
    let range_power_up_texture = load_texture("assets/blast-range-up.png").await.unwrap();
    range_power_up_texture.set_filter(FilterMode::Nearest);
    let bomb_count_power_up_texture = load_texture("assets/bomb-count-up.png").await.unwrap();
    bomb_count_power_up_texture.set_filter(FilterMode::Nearest);
    let mut player = Player {
        x: 1,
        y: 1,
        bomb_range: 2,
        max_bombs: 1,
        move_timer: 0.0,
    };
    let mut bombs: Vec<Bomb> = Vec::new();
    let mut explosions: Vec<Explosion> = Vec::new();
    let mut game_over = false;
    let mut enemies = vec![Enemy {
        x: COLS - 2,
        y: ROWS - 2,
        direction: Direction::Left,
        move_timer: 0.0,
    }];
    let mut game_won = false;
    let mut power_ups: Vec<PowerUp> = Vec::new();

    loop {
        let delta_time = get_frame_time();

        clear_background(BLACK);

        if (game_over || game_won) && is_key_pressed(KeyCode::R) {
            reset_game(
                &mut map,
                &mut player,
                &mut bombs,
                &mut explosions,
                &mut power_ups,
                &mut game_over,
                &mut enemies,
                &mut game_won,
            );
        }

        if !game_over && !game_won {
            handle_player_input(&map, &bombs, &mut player, delta_time);
            handle_bomb_input(&player, &mut bombs);

            collect_power_ups(&mut player, &mut power_ups);

            update_bombs(
                &mut map,
                &mut bombs,
                &mut explosions,
                &mut power_ups,
                delta_time,
            );
            update_enemies(&map, &bombs, &mut enemies, delta_time);

            remove_enemies_hit_by_explosions(&mut enemies, &explosions);

            if player_hit_by_explosion(&player, &explosions)
                || player_hit_by_enemy(&player, &enemies)
            {
                game_over = true;
            } else if enemies.is_empty() {
                game_won = true;
            }

            update_explosions(&mut explosions, delta_time);
        }

        draw_map(&map);
        draw_power_ups(
            &power_ups,
            &range_power_up_texture,
            &bomb_count_power_up_texture,
        );
        draw_bombs(&bombs, &bomb_texture);
        draw_explosions(&explosions);
        draw_enemies(&enemies);
        draw_player(&player, &player_texture);
        draw_hud(&player, &bombs, &enemies);

        if game_over {
            draw_game_over();
        }

        if game_won {
            draw_you_win();
        }

        next_frame().await;
    }
}

fn update_bombs(
    map: &mut [[Tile; COLS]; ROWS],
    bombs: &mut Vec<Bomb>,
    explosions: &mut Vec<Explosion>,
    power_ups: &mut Vec<PowerUp>,
    delta_time: f32,
) {
    // 每一幀都讓所有炸彈倒數
    for bomb in bombs.iter_mut() {
        bomb.timer -= delta_time;
    }

    let mut bombs_to_explode = Vec::new();
    let mut index = 0;

    while index < bombs.len() {
        if bombs[index].timer <= 0.0 {
            /*
             * bombs.remove(index) 會把炸彈從場上的炸彈列表拿走，避免同一顆炸彈重複爆炸
             * 沒有 index += 1，是因為 Vec 移除元素後，後面的元素會往前補位
             */
            bombs_to_explode.push(bombs.remove(index));
        } else {
            index += 1;
        }
    }

    let mut new_explosions = Vec::new();

    while let Some(bomb) = bombs_to_explode.pop() {
        let current_explosions = create_explosion_area(map, power_ups, bomb.x, bomb.y, bomb.range);

        queue_bombs_hit_by_explosions(bombs, &current_explosions, &mut bombs_to_explode);
        new_explosions.extend(current_explosions);
    }

    explosions.extend(new_explosions);
}

fn queue_bombs_hit_by_explosions(
    bombs: &mut Vec<Bomb>,
    explosions: &[Explosion],
    bombs_to_explode: &mut Vec<Bomb>,
) {
    let mut index = 0;

    while index < bombs.len() {
        let bomb_was_hit = explosions
            .iter()
            .any(|explosion| explosion.x == bombs[index].x && explosion.y == bombs[index].y);

        if bomb_was_hit {
            bombs_to_explode.push(bombs.remove(index));
        } else {
            index += 1;
        }
    }
}

fn update_explosions(explosions: &mut Vec<Explosion>, delta_time: f32) {
    for explosion in explosions.iter_mut() {
        explosion.timer -= delta_time;
    }

    explosions.retain(|explosion| explosion.timer > 0.0);
}

fn handle_bomb_input(player: &Player, bombs: &mut Vec<Bomb>) {
    if is_key_pressed(KeyCode::Space) {
        if bombs.len() >= player.max_bombs {
            return;
        }

        let already_has_bomb = bombs
            .iter()
            .any(|bomb| bomb.x == player.x && bomb.y == player.y);

        if !already_has_bomb {
            bombs.push(Bomb {
                x: player.x,
                y: player.y,
                timer: 2.0,
                range: player.bomb_range,
            });
        }
    }
}

fn handle_player_input(
    map: &[[Tile; COLS]; ROWS],
    bombs: &[Bomb],
    player: &mut Player,
    delta_time: f32,
) {
    if player.move_timer > 0.0 {
        player.move_timer -= delta_time;
    }

    if player.move_timer > 0.0 {
        return;
    }

    let movement = if is_key_down(KeyCode::Up) {
        Some((0, -1))
    } else if is_key_down(KeyCode::Down) {
        Some((0, 1))
    } else if is_key_down(KeyCode::Left) {
        Some((-1, 0))
    } else if is_key_down(KeyCode::Right) {
        Some((1, 0))
    } else {
        None
    };

    if let Some((dx, dy)) = movement {
        if try_move_player(map, bombs, player, dx, dy) {
            player.move_timer = PLAYER_MOVE_DELAY;
        }
    }
}

fn try_move_player(
    map: &[[Tile; COLS]; ROWS],
    bombs: &[Bomb],
    player: &mut Player,
    dx: i32,
    dy: i32,
) -> bool {
    let next_x = player.x as i32 + dx;
    let next_y = player.y as i32 + dy;

    if is_walkable(map, next_x, next_y) && !has_bomb_at(bombs, next_x, next_y) {
        player.x = next_x as usize;
        player.y = next_y as usize;
        true
    } else {
        false
    }
}

fn is_walkable(map: &[[Tile; COLS]; ROWS], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= COLS as i32 || y >= ROWS as i32 {
        return false;
    }

    matches!(map[y as usize][x as usize], Tile::Empty)
}

fn has_bomb_at(bombs: &[Bomb], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 {
        return false;
    }

    bombs
        .iter()
        .any(|bomb| bomb.x == x as usize && bomb.y == y as usize)
}

fn create_map() -> [[Tile; COLS]; ROWS] {
    let mut map = [[Tile::Empty; COLS]; ROWS];

    for y in 0..ROWS {
        for x in 0..COLS {
            if y == 0 || y == ROWS - 1 || x == 0 || x == COLS - 1 {
                map[y][x] = Tile::Wall;
            } else if x % 2 == 0 && y % 2 == 0 {
                map[y][x] = Tile::Wall;
            } else if (x + y) % 3 == 0 {
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

fn draw_player(player: &Player, texture: &Texture2D) {
    draw_texture_ex(
        texture,
        player.x as f32 * TILE_SIZE + 6.0,
        player.y as f32 * TILE_SIZE + 6.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(TILE_SIZE - 6.0, TILE_SIZE - 6.0)),
            ..Default::default()
        },
    );
}

fn draw_hud(player: &Player, bombs: &[Bomb], enemies: &[Enemy]) {
    let hud_y = ROWS as f32 * TILE_SIZE;
    let available_bombs = player.max_bombs.saturating_sub(bombs.len());
    let text = format!(
        "Range: {}   Bombs: {}/{}   Enemies: {}",
        player.bomb_range,
        available_bombs,
        player.max_bombs,
        enemies.len()
    );

    draw_rectangle(
        0.0,
        hud_y,
        COLS as f32 * TILE_SIZE,
        screen_height() - hud_y,
        Color::new(0.08, 0.08, 0.1, 1.0),
    );

    draw_text(&text, 16.0, hud_y + 34.0, 28.0, WHITE);
}

fn draw_bombs(bombs: &[Bomb], texture: &Texture2D) {
    for bomb in bombs {
        draw_texture_ex(
            texture,
            bomb.x as f32 * TILE_SIZE + 7.0,
            bomb.y as f32 * TILE_SIZE + 7.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(TILE_SIZE - 14.0, TILE_SIZE - 14.0)),
                ..Default::default()
            },
        );
    }
}

fn draw_explosions(explosions: &[Explosion]) {
    for explosion in explosions {
        draw_rectangle(
            explosion.x as f32 * TILE_SIZE + 4.0,
            explosion.y as f32 * TILE_SIZE + 4.0,
            TILE_SIZE - 8.0,
            TILE_SIZE - 8.0,
            ORANGE,
        )
    }
}

fn create_explosion_area(
    map: &mut [[Tile; COLS]; ROWS],
    power_ups: &mut Vec<PowerUp>,
    bomb_x: usize,
    bomb_y: usize,
    bomb_range: i32,
) -> Vec<Explosion> {
    let mut explosions = Vec::new();

    explosions.push(Explosion {
        x: bomb_x,
        y: bomb_y,
        timer: 0.4,
    });

    let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for (dx, dy) in directions {
        for distance in 1..=bomb_range {
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

                    // 炸掉磚塊後，有 30% 機率掉道具
                    if macroquad::rand::gen_range(0, 100) < 30 {
                        power_ups.push(PowerUp {
                            x: tile_x,
                            y: tile_y,
                            kind: random_power_up_kind(),
                        });
                    }

                    break;
                }
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
    let title = "GAME OVER";
    let hint = "Press R to restart";

    let title_size = 60.0;
    let hint_size = 30.0;

    let title_measure = measure_text(title, None, title_size as u16, 1.0);
    let hint_measure = measure_text(hint, None, hint_size as u16, 1.0);

    draw_text(
        title,
        screen_width() / 2.0 - title_measure.width / 2.0,
        screen_height() / 2.0 - 20.0,
        title_size,
        RED,
    );

    draw_text(
        hint,
        screen_width() / 2.0 - hint_measure.width / 2.0,
        screen_height() / 2.0 + 20.0,
        hint_size,
        WHITE,
    );
}

fn reset_game(
    map: &mut [[Tile; COLS]; ROWS],
    player: &mut Player,
    bombs: &mut Vec<Bomb>,
    explosions: &mut Vec<Explosion>,
    power_ups: &mut Vec<PowerUp>,
    game_over: &mut bool,
    enemies: &mut Vec<Enemy>,
    game_won: &mut bool,
) {
    *map = create_map();
    *player = Player {
        x: 1,
        y: 1,
        bomb_range: 2,
        max_bombs: 1,
        move_timer: 0.0,
    };
    bombs.clear();
    explosions.clear();
    power_ups.clear();

    *enemies = vec![Enemy {
        x: COLS - 2,
        y: ROWS - 2,
        direction: Direction::Left,
        move_timer: 0.0,
    }];

    *game_over = false;
    *game_won = false;
}

fn update_enemies(
    map: &[[Tile; COLS]; ROWS],
    bombs: &[Bomb],
    enemies: &mut Vec<Enemy>,
    delta_time: f32,
) {
    for enemy in enemies {
        enemy.move_timer -= delta_time;

        if enemy.move_timer > 0.0 {
            continue;
        }

        enemy.move_timer = 0.5;

        let (dx, dy) = direction_delta(enemy.direction);
        let next_x = enemy.x as i32 + dx;
        let next_y = enemy.y as i32 + dy;

        if is_walkable(map, next_x, next_y) && !has_bomb_at(bombs, next_x, next_y) {
            enemy.x = next_x as usize;
            enemy.y = next_y as usize;
        } else {
            enemy.direction = random_direction();
        }
    }
}

fn direction_delta(direction: Direction) -> (i32, i32) {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}

fn random_direction() -> Direction {
    match macroquad::rand::gen_range(0, 4) {
        0 => Direction::Up,
        1 => Direction::Down,
        2 => Direction::Left,
        _ => Direction::Right,
    }
}

fn draw_enemies(enemies: &[Enemy]) {
    for enemy in enemies {
        draw_rectangle(
            enemy.x as f32 * TILE_SIZE + 10.0,
            enemy.y as f32 * TILE_SIZE + 10.0,
            TILE_SIZE - 20.0,
            TILE_SIZE - 20.0,
            RED,
        );
    }
}

fn player_hit_by_enemy(player: &Player, enemies: &[Enemy]) -> bool {
    enemies
        .iter()
        .any(|enemy| enemy.x == player.x && enemy.y == player.y)
}

fn remove_enemies_hit_by_explosions(enemies: &mut Vec<Enemy>, explosions: &[Explosion]) {
    // 只留下符合條件的元素
    enemies.retain(|enemy| {
        /*
         * 如果沒有任何 explosion 跟 enemy 在同一格 -> 留下敵人
         * 如果有 explosion 跟 enemy 在同一格 -> 不留下，也就是移除敵人
         * 總結 : 保留所有沒有被爆炸打中的敵人
         */
        !explosions
            .iter()
            .any(|explosion| explosion.x == enemy.x && explosion.y == enemy.y)
    });
}

fn draw_you_win() {
    let title = "YOU WIN";
    let hint = "Press R to restart";

    let title_size = 60.0;
    let hint_size = 30.0;

    let title_measure = measure_text(title, None, title_size as u16, 1.0);
    let hint_measure = measure_text(hint, None, hint_size as u16, 1.0);

    draw_text(
        title,
        screen_width() / 2.0 - title_measure.width / 2.0,
        screen_height() / 2.0 - 20.0,
        title_size,
        GREEN,
    );

    draw_text(
        hint,
        screen_width() / 2.0 - hint_measure.width / 2.0,
        screen_height() / 2.0 + 30.0,
        hint_size,
        WHITE,
    );
}

fn random_power_up_kind() -> PowerUpKind {
    match macroquad::rand::gen_range(0, 2) {
        0 => PowerUpKind::Range,
        _ => PowerUpKind::Bomb,
    }
}

fn collect_power_ups(player: &mut Player, power_ups: &mut Vec<PowerUp>) {
    /*
     * 撿到的道具 -> 回傳 false -> 從 Vec 移除
     * 沒撿到的道具 -> 回傳 true -> 留在地圖上
     */
    power_ups.retain(|power_up| {
        if power_up.x == player.x && power_up.y == player.y {
            match power_up.kind {
                PowerUpKind::Range => player.bomb_range += 1,
                PowerUpKind::Bomb => player.max_bombs += 1,
            }

            false
        } else {
            true
        }
    });
}

// 畫道具
fn draw_power_ups(
    power_ups: &[PowerUp],
    range_texture: &Texture2D,
    bomb_count_texture: &Texture2D,
) {
    for power_up in power_ups {
        match power_up.kind {
            PowerUpKind::Range => {
                draw_texture_ex(
                    range_texture,
                    power_up.x as f32 * TILE_SIZE + 10.0,
                    power_up.y as f32 * TILE_SIZE + 10.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE - 20.0, TILE_SIZE - 20.0)),
                        ..Default::default()
                    },
                );
            }
            PowerUpKind::Bomb => {
                draw_texture_ex(
                    bomb_count_texture,
                    power_up.x as f32 * TILE_SIZE + 10.0,
                    power_up.y as f32 * TILE_SIZE + 10.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE - 20.0, TILE_SIZE - 20.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
