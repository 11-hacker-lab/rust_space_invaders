use macroquad::prelude::*;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const PLAYER_SPEED: f32 = 400.0;
const BULLET_SPEED: f32 = 500.0;
const ALIEN_ROWS: usize = 5;
const ALIEN_COLS: usize = 11;
const FIRE_COOLDOWN: f32 = 0.2;
const ALIEN_ART_ROWS: usize = 8;
const ALIEN_ART_COLS: usize = 11;
const ALIEN_PIXEL_ART: [&str; ALIEN_ART_ROWS] = [
    "00100000100",
    "00010001000",
    "00111111100",
    "01101110110",
    "11111111111",
    "10111111101",
    "10100000101",
    "00011011000",
];

#[derive(Clone, Copy, PartialEq)]
enum MenuTab {
    Main,
    Controls,
    Colors,
    Difficulty,
}

#[derive(Clone, Copy)]
struct ColorChoice {
    r: u8,
    g: u8,
    b: u8,
    label: &'static str,
}

impl ColorChoice {
    fn to_color(self) -> Color {
        Color::from_rgba(self.r, self.g, self.b, 255)
    }
}

const COLOR_OPTIONS: [ColorChoice; 6] = [
    ColorChoice { r: 255, g: 255, b: 255, label: "White" },
    ColorChoice { r: 255, g: 50,  b: 50,  label: "Red" },
    ColorChoice { r: 50,  g: 255, b: 50,  label: "Green" },
    ColorChoice { r: 50,  g: 200, b: 255, label: "Cyan" },
    ColorChoice { r: 255, g: 200, b: 50,  label: "Yellow" },
    ColorChoice { r: 200, g: 50,  b: 255, label: "Purple" },
];

#[derive(Clone, Copy, PartialEq)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    fn alien_speed(&self) -> f32 {
        match self {
            Difficulty::Easy   => 60.0,
            Difficulty::Normal => 100.0,
            Difficulty::Hard   => 160.0,
        }
    }
    fn shoot_chance(&self) -> u32 {
        match self {
            Difficulty::Easy   => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard   => 4,
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Difficulty::Easy   => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard   => "Hard",
        }
    }
}

struct Settings {
    key_left: KeyCode,
    key_right: KeyCode,
    key_fire: KeyCode,
    player_color_idx: usize,
    alien_color_idx: usize,
    bullet_player_color_idx: usize,
    bullet_alien_color_idx: usize,
    difficulty: Difficulty,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            key_left: KeyCode::Left,
            key_right: KeyCode::Right,
            key_fire: KeyCode::LeftControl,
            player_color_idx: 1,
            alien_color_idx: 0,
            bullet_player_color_idx: 1,
            bullet_alien_color_idx: 0,
            difficulty: Difficulty::Normal,
        }
    }
}

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
    Menu(MenuTab),
    Rebinding(usize),
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

fn check_collision(bullet: &Bullet, x: f32, y: f32, w: f32, h: f32) -> bool {
    bullet.x < x + w
        && bullet.x + bullet.width > x
        && bullet.y < y + h
        && bullet.y + bullet.height > y
}

fn reset_aliens(aliens: &mut Vec<Alien>) {
    aliens.clear();
    for row in 0..ALIEN_ROWS {
        for col in 0..ALIEN_COLS {
            aliens.push(Alien {
                x: 60.0 + col as f32 * 60.0,
                y: 60.0 + row as f32 * 45.0,
                width: 44.0,
                height: 32.0,
                alive: true,
            });
        }
    }
}

fn draw_alien(alien: &Alien, color: Color) {
    let pixel_w = alien.width / ALIEN_ART_COLS as f32;
    let pixel_h = alien.height / ALIEN_ART_ROWS as f32;
    for r in 0..ALIEN_ART_ROWS {
        for c in 0..ALIEN_ART_COLS {
            if ALIEN_PIXEL_ART[r].as_bytes()[c] == b'1' {
                draw_rectangle(
                    alien.x + c as f32 * pixel_w,
                    alien.y + r as f32 * pixel_h,
                    pixel_w,
                    pixel_h,
                    color,
                );
            }
        }
    }
}

fn key_name(k: KeyCode) -> &'static str {
    match k {
        KeyCode::Left         => "Left",
        KeyCode::Right        => "Right",
        KeyCode::LeftControl  => "LCtrl",
        KeyCode::RightControl => "RCtrl",
        KeyCode::Space        => "Space",
        KeyCode::Z            => "Z",
        KeyCode::X            => "X",
        KeyCode::A            => "A",
        KeyCode::D            => "D",
        _                     => "???",
    }
}

fn draw_tab_bar(active: MenuTab) {
    let tabs = [
        (MenuTab::Main,       "Main",      50.0),
        (MenuTab::Controls,   "Controls", 170.0),
        (MenuTab::Colors,     "Colors",   310.0),
        (MenuTab::Difficulty, "Difficulty",     430.0),

    ];
    for (tab, label, x) in &tabs {
        let color = if *tab == active { GREEN } else { GRAY };
        draw_text(label, *x, 60.0, 20.0, color);
    }
    draw_line(40.0, 70.0, 760.0, 70.0, 1.0, DARKGRAY);
    draw_text("< >", 720.0, 60.0, 16.0, Color::from_rgba(80,80,80,255));
}

const TAB_ORDER: [MenuTab; 4] = [MenuTab::Main, MenuTab::Controls, MenuTab::Colors, MenuTab::Difficulty];

fn tab_index(tab: MenuTab) -> usize {
    TAB_ORDER.iter().position(|t| *t == tab).unwrap_or(0)
}

fn tab_next(tab: MenuTab) -> MenuTab {
    TAB_ORDER[(tab_index(tab) + 1) % TAB_ORDER.len()]
}

fn tab_prev(tab: MenuTab) -> MenuTab {
    TAB_ORDER[(tab_index(tab) + TAB_ORDER.len() - 1) % TAB_ORDER.len()]
}

fn tab_clicked(mx: f32, my: f32) -> Option<MenuTab> {
    if my < 40.0 || my > 75.0 { return None; }
    if mx >= 80.0  && mx < 200.0 { return Some(MenuTab::Main); }
    if mx >= 230.0 && mx < 350.0 { return Some(MenuTab::Controls); }
    if mx >= 380.0 && mx < 500.0 { return Some(MenuTab::Colors); }
    if mx >= 530.0 && mx < 700.0 { return Some(MenuTab::Difficulty); }
    None
}

fn draw_color_row(label: &str, y: f32, selected: usize) {
    draw_text(label, 80.0, y, 20.0, WHITE);
    for (i, c) in COLOR_OPTIONS.iter().enumerate() {
        let bx = 260.0 + i as f32 * 70.0;
        let by = y - 16.0;
        draw_rectangle(bx, by, 55.0, 22.0, c.to_color());
        if i == selected {
            draw_rectangle_lines(bx - 2.0, by - 2.0, 59.0, 26.0, 2.0, YELLOW);
        }
    }
}

fn color_row_click(mx: f32, my: f32, row_y: f32) -> Option<usize> {
    if (my - row_y).abs() > 16.0 { return None; }
    for i in 0..COLOR_OPTIONS.len() {
        let bx = 260.0 + i as f32 * 70.0;
        if mx >= bx && mx < bx + 55.0 { return Some(i); }
    }
    None
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut settings = Settings::default();

    let mut player = Player {
        x: SCREEN_WIDTH / 2.0 - 25.0,
        y: SCREEN_HEIGHT - 50.0,
        width: 50.0,
        height: 20.0,
    };
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut shoot_cooldown_timer = 0.0f32;
    let mut aliens: Vec<Alien> = Vec::new();
    let mut alien_direction = 1.0f32;
    let mut alien_move_down = false;
    let mut score = 0u32;
    let mut game_state = GameState::Menu(MenuTab::Main);
    let mut color_row: usize = 0;
    let mut color_col: usize = 0;
    let mut diff_cursor: usize = 1;
    let mut color_tab_mode = false;

    loop {
        clear_background(BLACK);
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        match game_state {
            GameState::Menu(tab) => {
                draw_tab_bar(tab);

                // Tab switching: always on Main/Controls; on Difficulty always; on Colors only in tab_mode
                let allow_tab_switch = match tab {
                    MenuTab::Main | MenuTab::Controls | MenuTab::Difficulty => true,
                    MenuTab::Colors => color_tab_mode,
                };
                if allow_tab_switch {
                    if is_key_pressed(KeyCode::Right) {
                        color_tab_mode = false;
                        game_state = GameState::Menu(tab_next(tab));
                        next_frame().await;
                        continue;
                    }
                    if is_key_pressed(KeyCode::Left) {
                        color_tab_mode = false;
                        game_state = GameState::Menu(tab_prev(tab));
                        next_frame().await;
                        continue;
                    }
                }
                if clicked {
                    if let Some(t) = tab_clicked(mx, my) {
                        game_state = GameState::Menu(t);
                        next_frame().await;
                        continue;
                    }
                }

                match tab {
                    MenuTab::Main => {
                        draw_text("SPACE INVADERS", SCREEN_WIDTH / 2.0 - 180.0, SCREEN_HEIGHT / 2.0 - 60.0, 42.0, GREEN);
                        draw_text("Press ENTER to start", SCREEN_WIDTH / 2.0 - 140.0, SCREEN_HEIGHT / 2.0, 24.0, WHITE);
                        draw_text(&format!("Difficulty: {}", settings.difficulty.label()), SCREEN_WIDTH / 2.0 - 80.0, SCREEN_HEIGHT / 2.0 + 40.0, 20.0, GRAY);
                        if is_key_pressed(KeyCode::Enter) {
                            score = 0;
                            bullets.clear();
                            player.x = SCREEN_WIDTH / 2.0 - 25.0;
                            reset_aliens(&mut aliens);
                            alien_direction = 1.0;
                            alien_move_down = false;
                            shoot_cooldown_timer = 0.0;
                            game_state = GameState::Playing;
                        }
                    }

                    MenuTab::Controls => {
                        let bindings = [
                            ("Move Left",  settings.key_left),
                            ("Move Right", settings.key_right),
                            ("Fire",       settings.key_fire),
                        ];
                        for (i, (label, key)) in bindings.iter().enumerate() {
                            let y = 130.0 + i as f32 * 60.0;
                            draw_text(label, 100.0, y, 22.0, WHITE);
                            let bx = 360.0;
                            let by = y - 18.0;
                            draw_rectangle(bx, by, 120.0, 28.0, DARKGRAY);
                            draw_text(key_name(*key), bx + 10.0, y, 22.0, YELLOW);
                            draw_text("[Click mouse to rebind]", bx + 130.0, y, 18.0, GRAY);
                            if clicked && mx >= bx && mx < bx + 120.0 && my >= by && my < by + 28.0 {
                                game_state = GameState::Rebinding(i);
                            }
                        }
                    }

                    MenuTab::Colors => {
                        let n_rows = 4usize;
                        let n_cols = COLOR_OPTIONS.len();
                        if !color_tab_mode {
                            if is_key_pressed(KeyCode::Up) {
                                if color_row == 0 {
                                    color_tab_mode = true;
                                } else {
                                    color_row -= 1;
                                }
                            }
                            if is_key_pressed(KeyCode::Down) && color_row < n_rows - 1 { color_row += 1; }
                            if is_key_pressed(KeyCode::Left) && color_col > 0 { color_col -= 1; }
                            if is_key_pressed(KeyCode::Right) && color_col < n_cols - 1 { color_col += 1; }
                            if is_key_pressed(KeyCode::Enter) {
                                match color_row {
                                    0 => settings.player_color_idx = color_col,
                                    1 => settings.alien_color_idx = color_col,
                                    2 => settings.bullet_player_color_idx = color_col,
                                    _ => settings.bullet_alien_color_idx = color_col,
                                }
                            }
                        } else {
                            if is_key_pressed(KeyCode::Down) {
                                color_tab_mode = false;
                                color_row = 0;
                            }
                        }
                        let row_indices = [
                            settings.player_color_idx,
                            settings.alien_color_idx,
                            settings.bullet_player_color_idx,
                            settings.bullet_alien_color_idx,
                        ];
                        let labels = ["Player", "Alien", "Player Bullet", "Alien Bullet"];
                        for i in 0..n_rows {
                            let y = 130.0 + i as f32 * 70.0;
                            let row_color = if !color_tab_mode && i == color_row { YELLOW } else { WHITE };
                            draw_text(labels[i], 80.0, y, 20.0, row_color);
                            for (ci, c) in COLOR_OPTIONS.iter().enumerate() {
                                let bx = 260.0 + ci as f32 * 70.0;
                                let by = y - 16.0;
                                draw_rectangle(bx, by, 55.0, 22.0, c.to_color());
                                if ci == row_indices[i] {
                                    draw_rectangle_lines(bx - 2.0, by - 2.0, 59.0, 26.0, 2.0, WHITE);
                                }
                                if !color_tab_mode && i == color_row && ci == color_col {
                                    draw_rectangle_lines(bx - 3.0, by - 3.0, 61.0, 28.0, 2.0, YELLOW);
                                }
                            }
                            if clicked {
                                if let Some(ci) = color_row_click(mx, my, y) {
                                    color_tab_mode = false;
                                    color_row = i;
                                    color_col = ci;
                                    match i {
                                        0 => settings.player_color_idx = ci,
                                        1 => settings.alien_color_idx = ci,
                                        2 => settings.bullet_player_color_idx = ci,
                                        _ => settings.bullet_alien_color_idx = ci,
                                    }
                                }
                            }
                        }
                        let hint = if color_tab_mode { "Left/Right: switch tab   Down: back to colors" } else { "Up/Down: row   Left/Right: color   Enter: apply   Up on top row: tabs" };
                        draw_text(hint, 80.0, 430.0, 15.0, DARKGRAY);
                    }

                    MenuTab::Difficulty => {
                        let diffs = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard];
                        if is_key_pressed(KeyCode::Up) && diff_cursor > 0 { diff_cursor -= 1; }
                        if is_key_pressed(KeyCode::Down) && diff_cursor < 2 { diff_cursor += 1; }
                        if is_key_pressed(KeyCode::Enter) {
                            settings.difficulty = diffs[diff_cursor];
                        }
                        for (i, d) in diffs.iter().enumerate() {
                            let y = 160.0 + i as f32 * 70.0;
                            let is_cursor = i == diff_cursor;
                            let is_selected = settings.difficulty == *d;
                            let bg = if is_selected { Color::from_rgba(0,60,0,255) } else { DARKGRAY };
                            draw_rectangle(200.0, y - 22.0, 400.0, 40.0, bg);
                            if is_cursor {
                                draw_rectangle_lines(198.0, y - 24.0, 404.0, 44.0, 2.0, YELLOW);
                            }
                            let color = if is_selected { GREEN } else if is_cursor { YELLOW } else { GRAY };
                            draw_text(d.label(), 350.0, y, 26.0, color);
                            if clicked && mx >= 200.0 && mx < 600.0 && my >= y - 22.0 && my < y + 18.0 {
                                diff_cursor = i;
                                settings.difficulty = *d;
                            }
                        }
                        draw_text("Up/Down: navigate   Enter: apply   Left/Right: switch tab", 180.0, 390.0, 15.0, DARKGRAY);
                    }

                }
            }

            GameState::Rebinding(slot) => {
                draw_text("Press a key to bind...", 200.0, SCREEN_HEIGHT / 2.0, 28.0, YELLOW);
                draw_text("ESC to cancel", 280.0, SCREEN_HEIGHT / 2.0 + 40.0, 20.0, GRAY);
                let candidates = [
                    KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down,
                    KeyCode::Z, KeyCode::X, KeyCode::A, KeyCode::D,
                    KeyCode::Space, KeyCode::LeftControl, KeyCode::RightControl,
                ];
                for k in candidates {
                    if is_key_pressed(k) {
                        match slot {
                            0 => settings.key_left  = k,
                            1 => settings.key_right = k,
                            _ => settings.key_fire  = k,
                        }
                        game_state = GameState::Menu(MenuTab::Controls);
                        break;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Menu(MenuTab::Controls);
                }
            }

            GameState::Playing => {
                let delta = get_frame_time();

                if shoot_cooldown_timer > 0.0 {
                    shoot_cooldown_timer -= delta;
                }

                if is_key_down(settings.key_left) && player.x > 0.0 {
                    player.x -= PLAYER_SPEED * delta;
                }
                if is_key_down(settings.key_right) && player.x < SCREEN_WIDTH - player.width {
                    player.x += PLAYER_SPEED * delta;
                }
                if is_key_down(settings.key_fire) && shoot_cooldown_timer <= 0.0 {
                    bullets.push(Bullet {
                        x: player.x + player.width / 2.0 - 2.5,
                        y: player.y - 10.0,
                        width: 5.0,
                        height: 15.0,
                        from_player: true,
                    });
                    shoot_cooldown_timer = FIRE_COOLDOWN;
                }

                for b in bullets.iter_mut() {
                    if b.from_player {
                        b.y -= BULLET_SPEED * delta;
                    } else {
                        b.y += BULLET_SPEED * 0.6 * delta;
                    }
                }
                bullets.retain(|b| b.y > -b.height && b.y < SCREEN_HEIGHT);

                let total = (ALIEN_ROWS * ALIEN_COLS) as f32;
                let alive_count = aliens.iter().filter(|a| a.alive).count() as f32;
                let speed_factor = if alive_count / total >= 0.5 {
                    1.0
                } else {
                    let t = 1.0 - (alive_count / total) / 0.5;
                    1.0 + t * t * 1.5
                };
                let alien_speed = settings.difficulty.alien_speed() * speed_factor;
                let mut change_dir = false;
                for alien in aliens.iter().filter(|a| a.alive) {
                    if alien_direction > 0.0 && alien.x + alien.width >= SCREEN_WIDTH - 10.0 {
                        change_dir = true;
                        break;
                    }
                    if alien_direction < 0.0 && alien.x <= 10.0 {
                        change_dir = true;
                        break;
                    }
                }
                if change_dir {
                    alien_direction *= -1.0;
                    alien_move_down = true;
                }
                for alien in aliens.iter_mut().filter(|a| a.alive) {
                    if alien_move_down {
                        alien.y += 20.0;
                    } else {
                        alien.x += alien_speed * alien_direction * delta;
                    }
                    if alien.y + alien.height >= player.y {
                        game_state = GameState::GameOver(false);
                    }
                }
                alien_move_down = false;

                let shoot_chance = settings.difficulty.shoot_chance();
                if rand::rand() % 100 < shoot_chance {
                    let alive: Vec<usize> = aliens.iter().enumerate()
                        .filter(|(_, a)| a.alive)
                        .map(|(i, _)| i)
                        .collect();
                    if !alive.is_empty() {
                        let idx = alive[rand::rand() as usize % alive.len()];
                        let a = &aliens[idx];
                        bullets.push(Bullet {
                            x: a.x + a.width / 2.0 - 2.0,
                            y: a.y + a.height,
                            width: 4.0,
                            height: 12.0,
                            from_player: false,
                        });
                    }
                }

                let mut hit_player = false;
                for bullet in bullets.iter_mut() {
                    if bullet.from_player {
                        for alien in aliens.iter_mut().filter(|a| a.alive) {
                            if check_collision(bullet, alien.x, alien.y, alien.width, alien.height) {
                                alien.alive = false;
                                bullet.y = -100.0;
                                score += 10;
                                break;
                            }
                        }
                    } else if check_collision(bullet, player.x, player.y, player.width, player.height) {
                        hit_player = true;
                    }
                }
                if hit_player {
                    game_state = GameState::GameOver(false);
                }

                if aliens.iter().all(|a| !a.alive) {
                    game_state = GameState::GameOver(true);
                }

                let player_col  = COLOR_OPTIONS[settings.player_color_idx].to_color();
                let alien_col   = COLOR_OPTIONS[settings.alien_color_idx].to_color();
                let pbullet_col = COLOR_OPTIONS[settings.bullet_player_color_idx].to_color();
                let abullet_col = COLOR_OPTIONS[settings.bullet_alien_color_idx].to_color();

                draw_rectangle(player.x, player.y, player.width, player.height, player_col);

                for alien in aliens.iter().filter(|a| a.alive) {
                    draw_alien(alien, alien_col);
                }

                for b in &bullets {
                    let col = if b.from_player { pbullet_col } else { abullet_col };
                    draw_rectangle(b.x, b.y, b.width, b.height, col);
                }

                draw_text(&format!("SCORE: {}", score), 20.0, 30.0, 20.0, GREEN);
                draw_text(&format!("Difficulty: {}", settings.difficulty.label()), 600.0, 30.0, 18.0, GRAY);

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Menu(MenuTab::Main);
                }
            }

            GameState::GameOver(won) => {
                let text  = if won { "YOU WON!" } else { "GAME OVER" };
                let color = if won { GREEN } else { RED };
                draw_text(text, SCREEN_WIDTH / 2.0 - 140.0, SCREEN_HEIGHT / 2.0 - 50.0, 44.0, color);
                draw_text(&format!("Score: {}", score), SCREEN_WIDTH / 2.0 - 60.0, SCREEN_HEIGHT / 2.0, 22.0, WHITE);
                draw_text("ENTER — play again   ESC — menu", SCREEN_WIDTH / 2.0 - 200.0, SCREEN_HEIGHT / 2.0 + 50.0, 20.0, GRAY);

                if is_key_pressed(KeyCode::Enter) {
                    score = 0;
                    bullets.clear();
                    player.x = SCREEN_WIDTH / 2.0 - 25.0;
                    reset_aliens(&mut aliens);
                    alien_direction = 1.0;
                    alien_move_down = false;
                    shoot_cooldown_timer = 0.0;
                    game_state = GameState::Playing;
                }
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Menu(MenuTab::Main);
                }
            }
        }

        next_frame().await;
    }
}
