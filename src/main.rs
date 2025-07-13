use bracket_lib::prelude::*;
use lazy_static::lazy_static;
use image;
use image::GenericImageView; // 引入GenericImageView trait
use std::fs;
use std::path::Path;

// 游戏三种模式
enum GameMode {
    Menu,
    Playing,
    End,
}

// 背景样式
enum BackgroundStyle {
    Stars,
    Clouds,
    Mountains,
}

// 玩家样式
enum PlayerStyle {
    Dragon,
    Bird,
    Duck,
}

struct DefaultParameters {
    screen_width: i32,
    screen_height: i32,
    player_width: i32,
    player_height: i32,
    frame_duration: f32,
    obstacle_speed: f32,
    background_speed: f32,
}

lazy_static! {
    static ref DEFAULT_PARAMETERS: DefaultParameters = DefaultParameters {
        screen_width: 120,
        screen_height: 80,
        player_width: 14,
        player_height:14,
        frame_duration: 75.0,
        obstacle_speed: 0.5,
        background_speed: 0.001,
    };
}

struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    score: i32,
    obstacle_list: Vec<Obstacle>,
    background_offset: f32,
    distance: f32,
    menu_state: MenuState,
    settings: Settings,
    texture: Texture,
    high_score: i32, 
}

struct MenuState {
    current_menu: MainMenuOption,
    selected_option: i32,
    in_submenu: bool,
}

#[derive(PartialEq, Eq)]
enum MainMenuOption {
    Main,
    Background,
    Player,
    Obstacle,
}

struct Settings {
    background_style: BackgroundStyle,
    player_style: PlayerStyle,
    obstacle_distance: i32,
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

struct Obstacle {
    x: f32,
    gap_y: i32,
    size: i32,
    scored: bool, // 是否已得分
}

struct Texture {
    player_dragon: image::DynamicImage,
    player_bird: image::DynamicImage,
    player_duck: image::DynamicImage,
    background_stars: image::DynamicImage,
    background_clouds: image::DynamicImage,
    background_mountains: image::DynamicImage,
    menu_title: Vec<(i32, i32, FontCharType)>,
}

impl State {
    fn new() -> Self {
        let texture = Texture::new();

        let high_score = match fs::read_to_string("highscore.txt") {
        Ok(content) => content.trim().parse::<i32>().unwrap_or(0),
        Err(_) => 0,
    };
        Self {
            player: Player::new(2, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,
            score: 0,
            obstacle_list: vec![Obstacle::new(DEFAULT_PARAMETERS.screen_width, 0)],
            background_offset: 0.0,
            distance: 0.0,
            menu_state: MenuState {
                current_menu: MainMenuOption::Main,
                selected_option: 0,
                in_submenu: false,
            },
            settings: Settings {
                background_style: BackgroundStyle::Mountains,
                player_style: PlayerStyle::Duck,
                obstacle_distance: 50,
            },
            texture,
            high_score,
        }
    }

    fn update_background(&mut self, ctx: &mut BTerm) {
        self.background_offset += DEFAULT_PARAMETERS.background_speed * ctx.frame_time_ms;
        if self.background_offset > DEFAULT_PARAMETERS.screen_width as f32 {
            self.background_offset -= DEFAULT_PARAMETERS.screen_width as f32;
        }
    }

    fn playing(&mut self, ctx: &mut BTerm) {
        self.update_background(ctx);
        self.render_background(ctx); // 先渲染背景
        
        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > DEFAULT_PARAMETERS.frame_duration {
            self.player.gravity_to_move();
            self.frame_time = 0.0;
        }

        // 按下空格键时飞起
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        // 渲染玩家
        self.player.render(ctx, &self.texture, &self.settings.player_style);

        // 显示分数和提示
        ctx.print(0, 0, "Press Space to flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        // 渲染障碍物
        for obstacle in &mut self.obstacle_list {
            obstacle.render(ctx);

            if self.player.x > obstacle.x as i32 && !obstacle.scored {
                self.score += 1;
                obstacle.scored = true; // 标记已得分
            }

            if obstacle.hit_obstacle(&self.player) {
                self.mode = GameMode::End;
            }
        }
        
        self.obstacle_list.retain(|obstacle| obstacle.x > 0.0);
        self.distance += DEFAULT_PARAMETERS.obstacle_speed;

        if(self.distance > self.settings.obstacle_distance as f32){
            self.obstacle_list.push(Obstacle::new(
                DEFAULT_PARAMETERS.screen_width,
                self.score
            ));
            self.distance = 0.0;
        }

        // 判断是否碰到边界或障碍物
        if self.player.y + DEFAULT_PARAMETERS.player_height > DEFAULT_PARAMETERS.screen_height   {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        
        self.player = Player::new(2, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.score = 0;
        self.distance = 0.0;
        self.obstacle_list = vec![Obstacle::new(
            DEFAULT_PARAMETERS.screen_width,
            self.score
        )];
        self.background_offset = 0.0;
    }

    fn end(&mut self, ctx: &mut BTerm) {
        // 更新历史最高分（如果当前分数更高）
        if self.score > self.high_score {
            self.high_score = self.score;
            let _ = fs::write("highscore.txt", self.high_score.to_string()); // 保存到文件
        }
        self.update_background(ctx);
        self.render_background(ctx); // 渲染背景

        ctx.print_color_centered(5, WHITE,BLACK,"You are dead!");
        ctx.print_color_centered(6, WHITE,BLACK,&format!("Final Score: {}", self.score));
        ctx.print_color_centered(7,WHITE,BLACK, &format!("High Score: {}", self.high_score)); // 显示历史最高分
        ctx.print_color_centered(8,WHITE,BLACK, "(P) Play Again");
        ctx.print_color_centered(9, WHITE,BLACK,"(M) Main Menu");
        ctx.print_color_centered(10, WHITE,BLACK,"(Q) Quit Game");

        // 判断按键
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::M => self.mode = GameMode::Menu,
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        self.render_background(ctx); // 渲染背景
        self.update_background(ctx);
        // 渲染标题
        for (x, y, char) in &self.texture.menu_title {
            ctx.set(*x, *y, YELLOW, RGBA::from_u8(0,0,0,0), *char);
        }

        // 根据当前菜单状态渲染不同的菜单
        match self.menu_state.current_menu {
            MainMenuOption::Main => self.render_main_menu(ctx),
            MainMenuOption::Background => self.render_background_menu(ctx),
            MainMenuOption::Player => self.render_player_menu(ctx),
            MainMenuOption::Obstacle => self.render_obstacle_menu(ctx),
        }

        // 处理菜单导航
        self.handle_menu_input(ctx);
    }

    fn render_main_menu(&mut self, ctx: &mut BTerm) {
        let options = vec![
            "Start Game",
            "Background Style",
            "Player Style",
            "Obstacle Distance",
            "Quit Game",
        ];

        for (i, option) in options.iter().enumerate() {
            let color = if i as i32 == self.menu_state.selected_option {
                YELLOW
            } else {
                WHITE
            };

            ctx.print_color_centered(15 + i * 2, color, RGBA::from_u8(0,0,0,0), option);
        }
    }

    fn render_background_menu(&mut self, ctx: &mut BTerm) {
        let options = vec![
            "Stars",
            "Clouds",
            "Mountains",
            "Back",
        ];

        ctx.print_color_centered(12, WHITE,BLACK,"Select Background Style");

        for (i, option) in options.iter().enumerate() {
            let color = if i as i32 == self.menu_state.selected_option {
                YELLOW
            } else {
                WHITE
            };

            let is_active = match (i, &self.settings.background_style) {
                (0, BackgroundStyle::Stars) => "(*) ",
                (1, BackgroundStyle::Clouds) => "(*) ",
                (2, BackgroundStyle::Mountains) => "(*) ",
                _ => "( ) ",
            };

            ctx.print_color_centered(15 + i * 2, color, RGBA::from_u8(0,0,0,0), &format!("{}{}", is_active, option));
        }
    }

    fn render_player_menu(&mut self, ctx: &mut BTerm) {
        let options = vec![
            "Dragon",
            "Bird",
            "Duck",
            "Back",
        ];

        ctx.print_color_centered(12, WHITE, BLACK, "Select Player Style");

        for (i, option) in options.iter().enumerate() {
            let color = if i as i32 == self.menu_state.selected_option {
                YELLOW
            } else {
                WHITE
            };

            let is_active = match (i, &self.settings.player_style) {
                (0, PlayerStyle::Dragon) => "(*) ",
                (1, PlayerStyle::Bird) => "(*) ",
                (2, PlayerStyle::Duck) => "(*) ",
                _ => "( ) ",
            };

            ctx.print_color_centered(15 + i * 2, color, RGBA::from_u8(0,0,0,0), &format!("{}{}", is_active, option));
        }
    }

    fn render_obstacle_menu(&mut self, ctx: &mut BTerm) {
        ctx.print_centered(12, "Obstacle Distance");
        ctx.print_centered(14, &format!("Current: {} spaces", self.settings.obstacle_distance));
        ctx.print_centered(16, "(Use Left/Right to adjust)");
        ctx.print_centered(18, "Back");
    }

    fn handle_menu_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Up => {
                    if self.menu_state.selected_option > 0 {
                        self.menu_state.selected_option -= 1;
                    }
                }
                VirtualKeyCode::Down => {
                    let max_options = match self.menu_state.current_menu {
                        MainMenuOption::Main => 4,
                        MainMenuOption::Background => 3,
                        MainMenuOption::Player => 3,
                        MainMenuOption::Obstacle => 1,
                    };

                    if self.menu_state.selected_option < max_options {
                        self.menu_state.selected_option += 1;
                    }
                }
                VirtualKeyCode::Return => {
                    match self.menu_state.current_menu {
                        MainMenuOption::Main => {
                            match self.menu_state.selected_option {
                                0 => self.restart(), // 开始游戏
                                1 => {
                                    self.menu_state.current_menu = MainMenuOption::Background;
                                    self.menu_state.selected_option = 0;
                                }
                                2 => {
                                    self.menu_state.current_menu = MainMenuOption::Player;
                                    self.menu_state.selected_option = 0;
                                }
                                3 => {
                                    self.menu_state.current_menu = MainMenuOption::Obstacle;
                                    self.menu_state.selected_option = 0;
                                }
                                4 => ctx.quitting = true, // 退出游戏
                                _ => {}
                            }
                        }
                        MainMenuOption::Background => {
                            match self.menu_state.selected_option {
                                0 => self.settings.background_style = BackgroundStyle::Stars,
                                1 => self.settings.background_style = BackgroundStyle::Clouds,
                                2 => self.settings.background_style = BackgroundStyle::Mountains,
                                3 => {
                                    self.menu_state.current_menu = MainMenuOption::Main;
                                    self.menu_state.selected_option = 1;
                                }
                                _ => {}
                            }
                        }
                        MainMenuOption::Player => {
                            match self.menu_state.selected_option {
                                0 => self.settings.player_style = PlayerStyle::Dragon,
                                1 => self.settings.player_style = PlayerStyle::Bird,
                                2 => self.settings.player_style = PlayerStyle::Duck,
                                3 => {
                                    self.menu_state.current_menu = MainMenuOption::Main;
                                    self.menu_state.selected_option = 2;
                                }
                                _ => {}
                            }
                        }
                        MainMenuOption::Obstacle => {
                            if self.menu_state.selected_option == 1 {
                                self.menu_state.current_menu = MainMenuOption::Main;
                                self.menu_state.selected_option = 3;
                            }
                        }
                    }
                }
                VirtualKeyCode::Left => {
                    if self.menu_state.current_menu == MainMenuOption::Obstacle && 
                       self.menu_state.selected_option == 0 {
                        self.settings.obstacle_distance = i32::max(40, self.settings.obstacle_distance - 5);
                    }
                }
                VirtualKeyCode::Right => {
                    if (self.menu_state.current_menu == MainMenuOption::Obstacle) && 
                       self.menu_state.selected_option == 0 {
                        self.settings.obstacle_distance = i32::min(60, self.settings.obstacle_distance + 5);
                    }
                }
                VirtualKeyCode::Escape => {
                    self.menu_state.current_menu = MainMenuOption::Main;
                    self.menu_state.selected_option = 0;
                }
                _ => {}
            }
        }
    }

    fn render_background(&self, ctx: &mut BTerm) {
        match self.settings.background_style {
            BackgroundStyle::Stars => self.render_stars_background(ctx),
            BackgroundStyle::Clouds => self.render_clouds_background(ctx),
            BackgroundStyle::Mountains => self.render_mountains_background(ctx),
        }
    }

    fn render_stars_background(&self, ctx: &mut BTerm) {
        self.render_looping_background(ctx, &self.texture.background_stars);
    }

    fn render_clouds_background(&self, ctx: &mut BTerm) {
        self.render_looping_background(ctx, &self.texture.background_clouds);
    }

    fn render_mountains_background(&self, ctx: &mut BTerm) {
        self.render_looping_background(ctx, &self.texture.background_mountains);
    }

    fn render_looping_background(&self, ctx: &mut BTerm, background: &image::DynamicImage) {
        let width = background.width() as i32;
        let height = background.height() as i32;
        let offset = self.background_offset as i32 % width; // 确保偏移量在合理范围内

        for y in 0..DEFAULT_PARAMETERS.screen_height {
            for x in 0..DEFAULT_PARAMETERS.screen_width {
                let bg_x = (x + offset) % width;
                let bg_y = y % height;
                
                let pixel = background.get_pixel(bg_x as u32, bg_y as u32);
                let color = RGB::from_u8(pixel[0], pixel[1], pixel[2]);
                ctx.set(x, y, BLACK,color, to_cp437(' '));
            }
        }
    }
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            velocity: 0.0,
        }
    }

fn render(&mut self, ctx: &mut BTerm, texture: &Texture, style: &PlayerStyle) {
    let image = match style {
        PlayerStyle::Dragon => &texture.player_dragon,
        PlayerStyle::Bird => &texture.player_bird,
        PlayerStyle::Duck => &texture.player_duck,
    };

    for y in 0..DEFAULT_PARAMETERS.player_height {
        for x in 0..DEFAULT_PARAMETERS.player_width {
            let screen_x = self.x + x;
            let screen_y = self.y + y;

            if screen_x < 0 || screen_x >= DEFAULT_PARAMETERS.screen_width ||
               screen_y < 0 || screen_y >= DEFAULT_PARAMETERS.screen_height {
                continue;
            }

            let pixel = image.get_pixel(x as u32, y as u32);
            let alpha = pixel[3];

            if alpha == 0 {
                continue; // 透明像素不渲染
            }

            let color = RGB::from_u8(pixel[0], pixel[1], pixel[2]);
            ctx.set(screen_x, screen_y, BLACK, color, to_cp437(' '));
        }
    }
}

    fn gravity_to_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity as i32;
        

        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.5;
    }
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Self {
            x: x as f32,
            gap_y: random.range(30, 60),
            size: i32::max(20, 40 - score / 2),
            scored: false, // 初始状态未得分
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        self.x -= DEFAULT_PARAMETERS.obstacle_speed;
        let half_size = self.size / 2;

        // 绘制上半部分障碍物
        for y in 0..self.gap_y - half_size {
            ctx.set(self.x as i32, y, RED, YELLOW, to_cp437('|'));
        }

        // 绘制下半部分障碍物
        for y in self.gap_y + half_size..DEFAULT_PARAMETERS.screen_height {
            ctx.set(self.x as i32, y, RED, YELLOW, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let player_left_gap = player.x < self.x as i32;
        let player_right_gap = (player.x +DEFAULT_PARAMETERS.player_width) > self.x as i32;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = (player.y +DEFAULT_PARAMETERS.player_height) > self.gap_y + half_size;
        (player_left_gap && player_right_gap) && (player_above_gap || player_below_gap)
    }
}

impl Texture {
    fn new() -> Self {
        // 玩家贴图
        let player_dragon = image::load_from_memory(include_bytes!("player/dragon.png"))
            .expect("Failed to load dragon image");
        let player_bird = image::load_from_memory(include_bytes!("player/bird.png"))
            .expect("Failed to load bird image");
        let player_duck = image::load_from_memory(include_bytes!("player/duck.png"))
            .expect("Failed to load duck image");

        // 背景贴图
        let background_stars = image::load_from_memory(include_bytes!("background/stars.png"))
            .expect("Failed to load stars background image");
        let background_clouds = image::load_from_memory(include_bytes!("background/clouds.png"))
            .expect("Failed to load clouds background image");
        let background_mountains = image::load_from_memory(include_bytes!("background/mountains.png"))
            .expect("Failed to load mountains background image");

        // 菜单标题
        let menu_title = vec![
            (25, 5, to_cp437('F')), (27, 5, to_cp437('L')), (29, 5, to_cp437('A')),
            (31, 5, to_cp437('P')), (33, 5, to_cp437('P')), (35, 5, to_cp437('Y')),

            (23, 7, to_cp437('D')), (25, 7, to_cp437('R')), (27, 7, to_cp437('A')),
            (29, 7, to_cp437('G')), (31, 7, to_cp437('O')), (33, 7, to_cp437('N')),
        ];
        Self {
            player_dragon,
            player_bird,
            player_duck,
            background_stars,
            background_clouds,
            background_mountains,
            menu_title,
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.playing(ctx),
            GameMode::End => self.end(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple(DEFAULT_PARAMETERS.screen_width, DEFAULT_PARAMETERS.screen_height)?
        .with_tile_dimensions(10,10)   
        .with_title("Flappy Animals")
        .build()?;
    main_loop(context, State::new())
}