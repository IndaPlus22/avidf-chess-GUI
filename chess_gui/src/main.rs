/**
 * Chess GUI template.
 * Author: Isak Larsson <isaklar@kth.se>
 * Last updated: 2022-09-29
 */
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::collections::HashMap;

use chess_template::{Colour, Game, PieceType, Piece, Position, GameState};

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, MouseButton, MouseCursorEvent, PressEvent, ReleaseEvent};

/// A chess board is 8x8 tiles.
const GRID_SIZE: i16 = 8;
/// Sutible size of each tile.
const GRID_CELL_SIZE: (i16, i16) = (90, 90);

/// Size of the application window.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE as f32 * GRID_CELL_SIZE.0 as f32 + 340.0,
    GRID_SIZE as f32 * GRID_CELL_SIZE.1 as f32,
);

// GUI Color representations
const BLACK: [f32; 4] = [228.0 / 255.0, 196.0 / 255.0, 108.0 / 255.0, 1.0];
const WHITE: [f32; 4] = [188.0 / 255.0, 140.0 / 255.0, 76.0 / 255.0, 1.0];

pub struct App {
    gl: GlGraphics,                                 // OpenGL drawing backend.
    mouse_pos: [f64; 2],                            // Current mouse postition
    sprites: HashMap<Piece, Texture>, // For easy access to the apropriate PNGs
    game: Game, // Save piece positions, which tiles has been clicked, current colour, etc...

    left_click: bool, //checks if the left key is being pressed 
    move_piece: Option<(i16,i16)>, //gives the coordinates of a piece when moved
}

impl App {
    fn new(opengl: OpenGL) -> App {
        App {
            gl: GlGraphics::new(opengl),
            mouse_pos: [0., 0.],
            game: Game::new(),
            sprites: Self::load_sprites(),

            left_click: false,
            move_piece: None,
        }
    }

    fn render(&mut self, args: &RenderArgs, glyphs: &mut GlyphCache) {
        use graphics::*; // Now we don't have to use this everytime :D

        let square = rectangle::square(0.0, 0.0, GRID_CELL_SIZE.0 as f64);
        let board = self.game.get_board();
        let mouse_position = self.on_tile();

        self.gl.draw(args.viewport(), |c, gl| {
            //clear the screen
            clear([0.0, 0.127, 0.256, 0.52], gl);
            // clear(GREEN, gl);
            // Draw grid
            for row in 0..8 {
                for col in 0..8 {
                    rectangle(
                        match col % 2 {
                            0 => {
                                if row % 2 == 0 {
                                    BLACK
                                } else {
                                    WHITE
                                }
                            }
                            _ => {
                                if row % 2 == 0 {
                                    WHITE
                                } else {
                                    BLACK
                                }
                            }
                        },
                        square,
                        c.transform.trans(
                            (col * GRID_CELL_SIZE.0) as f64,
                            (row * GRID_CELL_SIZE.1) as f64,
                        ),
                        gl,
                    );

                }
            }

            // Draw pieces
            // I have to rewrite the for loops otherwise the white pices go under
            for row in 0..8 {
                for col in 0..8 {
                    if let Some(piece) = board[(row * 8 + col) as usize] {
                        let img = Image::new().rect(square);

                        let correct_tile = mouse_position == (col, row);
                        let piece_moving = self.move_piece.is_some() && self.move_piece.unwrap() == (col, row);
                        
                        if (self.left_click && correct_tile) || piece_moving { 
                            self.left_click = false;
                            self.move_piece = Some((col, row));                     
                            
                            // Follow mouse
                            img.draw(
                                self.sprites.get(&piece).unwrap(),
                                &c.draw_state,
                                c.transform.trans(
                                    self.mouse_pos[0] - GRID_CELL_SIZE.0 as f64 / 2.0,
                                    self.mouse_pos[1] - GRID_CELL_SIZE.1 as f64 / 2.0,
                                ),
                                gl,
                            );
                        } else {
                            img.draw(
                                self.sprites.get(&piece).unwrap(),
                                &c.draw_state,
                                c.transform.trans(
                                    (col * GRID_CELL_SIZE.0) as f64,
                                    (row * GRID_CELL_SIZE.0) as f64,
                                ),
                                gl,
                            );
                        }
                    }
                }
            }

            // Game state
            let state_text = format!("Game state: {:?}", self.game.get_game_state());
            let state_text_postition = c.transform.trans(
                (SCREEN_SIZE.0 - 320.0) as f64,
                (SCREEN_SIZE.1 - 10.0) as f64,
            );
            Text::new_color([0.01, 0.500, 0.334, 0.8], 24)
            .draw(&state_text, glyphs, &c.draw_state, state_text_postition, gl).unwrap();
            
            //Player turn
            let turn_text = format!("Turn: {:?}", self.game.get_active_colour());
            let turn_text_postition = c.transform.trans(
            (SCREEN_SIZE.0 - 320.0) as f64,
            (SCREEN_SIZE.1 - 40.0) as f64,
            );
            Text::new_color([0.01, 0.500, 0.334, 0.8], 24)
            .draw(&turn_text, glyphs, &c.draw_state, turn_text_postition, gl).unwrap();
            
            // Announce winner
            if self.game.get_game_state() == GameState::GameOver {
                let gameover_text = format!("{:?} is the winner,\n and the winner takes it all!", self.game.get_active_colour());
                let gameover_text_size: (f32, f32) = ((22 * gameover_text.len()) as f32, 24.0);
                let gameover_text_postition = c.transform.trans(
                    (SCREEN_SIZE.0 / 2.0 - gameover_text_size.0 / 2.0) as f64,
                    (SCREEN_SIZE.1 / 2.0 - gameover_text_size.1 / 2.0) as f64,
                );
                Text::new_color([0.01, 0.500, 0.334, 0.8], 45)
                    .draw(&gameover_text, glyphs, &c.draw_state, gameover_text_postition, gl).unwrap();
            }

        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Currently empty, but maybe you can find a fun use for it!
    }

    #[rustfmt::skip]
    /// Loads chess piceses images into vector.
    fn load_sprites() -> HashMap<Piece, Texture> {
        use Colour::*;
        use PieceType::*;
        [
            (Piece { colour: Black, piece_type: King }, "resources/black_king.png".to_string()),
            (Piece { colour: Black, piece_type: Queen }, "resources/black_queen.png".to_string()),
            (Piece { colour: Black, piece_type: Rook }, "resources/black_rook.png".to_string()),
            (Piece { colour: Black, piece_type: Pawn }, "resources/black_pawn.png".to_string()),
            (Piece { colour: Black, piece_type: Bishop }, "resources/black_bishop.png".to_string()),
            (Piece { colour: Black, piece_type: Knight }, "resources/black_knight.png".to_string()),
            (Piece { colour: White, piece_type: King }, "resources/white_king.png".to_string()),
            (Piece { colour: White, piece_type: Queen }, "resources/white_queen.png".to_string()),
            (Piece { colour: White, piece_type: Rook }, "resources/white_rook.png".to_string()),
            (Piece { colour: White, piece_type: Pawn }, "resources/white_pawn.png".to_string()),
            (Piece { colour: White, piece_type: Bishop }, "resources/white_bishop.png".to_string()),
            (Piece { colour: White, piece_type: Knight }, "resources/white_knight.png".to_string())
        ]
            .iter()
            .map(|(piece, path)| {
                (*piece, Texture::from_path(path, &TextureSettings::new()).unwrap())
            })
            .collect::<HashMap<Piece, Texture>>()
    }

    /// Returns which cell the mouse is in
    fn on_tile(&self) -> (i16, i16) {
        (
            (self.mouse_pos[0] / GRID_CELL_SIZE.0 as f64).floor() as i16, 
            (self.mouse_pos[1] / GRID_CELL_SIZE.1 as f64).floor() as i16
        )
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window =
        WindowSettings::new("Chess", [SCREEN_SIZE.0 as f64, SCREEN_SIZE.1 as f64])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Initialize our app state
    let mut app = App::new(opengl);

    // Initialize font
    let mut glyphs = GlyphCache::new(
        "resources/AbyssinicaSIL-Regular.ttf",
        (),
        TextureSettings::new(),
    )
    .unwrap();

    let mut events = Events::new(EventSettings::new());
    // Our "game loop". Will run until we exit the window
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &mut glyphs);
        }
        if let Some(args) = e.update_args() {
            app.update(&args);
        }
        if let Some(pos) = e.mouse_cursor_args() {
            app.mouse_pos = pos;
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            app.left_click = true;
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            // Follows the piece when selected
            //Now with hold
            if let Some(pos) = app.move_piece {
                let mouse_pos = app.on_tile();
                
                let from = Position::new(pos.1 as usize, pos.0 as usize).ok().unwrap();
                let to = Position::new(mouse_pos.1 as usize, mouse_pos.0 as usize).ok().unwrap();

                app.game.make_move_pos(from, to);
                app.move_piece = None;
            }
        }
    }
}