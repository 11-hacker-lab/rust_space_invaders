use macroquad::prelude::*;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const PLAYER_SPEED: f32 = 400.0;
const BULLET_SPEED: f32 = 500.0;
const ALIEN_ROWS: usize = 5;
const ALIEN_COLS: usize = 11;
const ALIEN_SPEED_X: f32 = 100.0;
const FIRE_COOLDOWN: f32 = 0.2;
struct Player {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct Bullet {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    from_player: bool,
}

struct Alien {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    alive: bool,
}

enum GameState {
    Menu,
    Playing,
    GameOver(bool),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Rust Space Invaders".to_string(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player {
        x: SCREEN_WIDTH / 2.0 - 25.0,
        y: SCREEN_HEIGHT - 50.0,
        width: 50.0,
        height: 20.0,
    };

    let mut bullets: Vec<Bullet> = Vec::new();
    let mut shoot_cooldown_timer = 0.0;
    let mut aliens: Vec<Alien> = Vec::new();
    reset_aliens(&mut aliens);

    let mut alien_direction = 1.0;
    let mut alien_move_down = false;
    let mut score = 0;
    let mut game_state = GameState::Menu;

    loop {
        clear_background(BLACK);

        match game_state {
            GameState::Menu => {
                draw_text("SPACE INVADERS", SCREEN_WIDTH / 2.0 - 180.0, SCREEN_HEIGHT / 2.0 - 50.0, 40.0, GREEN);
                draw_text("Press ENTER to start game", SCREEN_WIDTH / 2.0 - 200.0, SCREEN_HEIGHT / 2.0 + 10.0, 20.0, WHITE);
                
                if is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::Playing;
                    score = 0;
                    bullets.clear();
                    reset_aliens(&mut aliens);
                }
            }
            
            GameState::Playing => {
                let delta_time = get_frame_time();

if shoot_cooldown_timer > 0.0 {
    shoot_cooldown_timer -= delta_time;
}

if is_key_down(KeyCode::Left) && player.x > 0.0 {
    player.x -= PLAYER_SPEED * delta_time;
}
if is_key_down(KeyCode::Right) && player.x < SCREEN_WIDTH - player.width {
    player.x += PLAYER_SPEED * delta_time;
}

if is_key_down(KeyCode::LeftControl) && shoot_cooldown_timer <= 0.0 {
    bullets.push(Bullet {
        x: player.x + player.width / 2.0 - 2.5,
        y: player.y - 10.0,
        width: 5.0,
        height: 15.0,
        from_player: true,
    });
    
    shoot_cooldown_timer = FIRE_COOLDOWN;
}

                for bullet in bullets.iter_mut() {
                    if bullet.from_player {
                        bullet.y -= BULLET_SPEED * delta_time;
                    } else {
                        bullet.y += (BULLET_SPEED * 0.6) * delta_time;
                    }
                }
                bullets.retain(|b| b.y > 0.0 && b.y < SCREEN_HEIGHT);

                let mut change_dir = false;
                for alien in aliens.iter().filter(|a| a.alive) {
                    if alien_direction > 0.0 && alien.x >= SCREEN_WIDTH - alien.width - 10.0 {
                        change_dir = true;
                    }
                    if alien_direction < 0.0 && alien.x <= 10.0 {
                        change_dir = true;
                    }
                    if alien.y + alien.height >= player.y {
                        game_state = GameState::GameOver(false);
                    }
                }

                if change_dir {
                    alien_direction *= -1.0;
                    alien_move_down = true;
                }

                for alien in aliens.iter_mut().filter(|a| a.alive) {
                    if alien_move_down {
                        alien.y += 30.0;
                    } else {
                        alien.x += ALIEN_SPEED_X * alien_direction * delta_time;
                    }
                }
                alien_move_down = false;

                if rand::rand() % 100 < 2 {
                    let alive_aliens: Vec<&Alien> = aliens.iter().filter(|a| a.alive).collect();
                    if !alive_aliens.is_empty() {
                        let random_alien = alive_aliens[rand::rand() as usize % alive_aliens.len()];
                        bullets.push(Bullet {
                            x: random_alien.x + random_alien.width / 2.0,
                            y: random_alien.y + random_alien.height,
                            width: 4.0,
                            height: 12.0,
                            from_player: false,
                        });
                    }
                }

                for bullet in bullets.iter_mut() {
                    if bullet.from_player {
                        for alien in aliens.iter_mut().filter(|a| a.alive) {
                            if check_collision(bullet, alien.x, alien.y, alien.width, alien.height) {
                                alien.alive = false;
                                bullet.y = -100.0;
                                score += 10;
                            }
                        }
                    } else {
                        if check_collision(bullet, player.x, player.y, player.width, player.height) {
                            game_state = GameState::GameOver(false);
                        }
                    }
                }

                if aliens.iter().all(|a| !a.alive) {
                    game_state = GameState::GameOver(true);
                }

                draw_rectangle(player.x, player.y, player.width, player.height, GREEN);
                
                for alien in aliens.iter().filter(|a| a.alive) {
                    draw_rectangle(alien.x, alien.y, alien.width, alien.height, WHITE);
                }

                for bullet in &bullets {
                    let color = if bullet.from_player { RED } else { YELLOW };
                    draw_rectangle(bullet.x, bullet.y, bullet.width, bullet.height, color);
                }

                draw_text(&format!("SCORE: {}", score), 20.0, 30.0, 20.0, GREEN);
            }
            
            GameState::GameOver(won) => {
                let text = if won { "YOU WON!" } else { "GAME OVER" };
                let color = if won { GREEN } else { RED };
                
                draw_text(text, SCREEN_WIDTH / 2.0 - 140.0, SCREEN_HEIGHT / 2.0 - 50.0, 40.0, color);
                draw_text(&format!("Your count: {}", score), SCREEN_WIDTH / 2.0 - 70.0, SCREEN_HEIGHT / 2.0, 20.0, WHITE);
                draw_text("Press ESC to exit", SCREEN_WIDTH / 2.0 - 180.0, SCREEN_HEIGHT / 2.0 + 40.0, 20.0, GRAY);

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Menu;
                }
            }
        }

        next_frame().await
    }
}

fn reset_aliens(aliens: &mut Vec<Alien>) {
    aliens.clear();
    for row in 0..ALIEN_ROWS {
        for col in 0..ALIEN_COLS {
            aliens.push(Alien {
                x: 80.0 + col as f32 * 55.0,
                y: 60.0 + row as f32 * 40.0,
                width: 35.0,
                height: 25.0,
                alive: true,
            });
        }
    }
}

fn check_collision(bullet: &Bullet, x: f32, y: f32, w: f32, h: f32) -> bool {
    bullet.x < x + w &&
    bullet.x + bullet.width > x &&
    bullet.y < y + h &&
    bullet.y + bullet.height > y
}
