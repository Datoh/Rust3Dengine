use tetra::{window, input, graphics};
use tetra::graphics::{Color, DrawParams, Texture};
use tetra::{Context, ContextBuilder, State};
use tetra::math::Vec2;

pub fn draw_line(ctx: &mut Context, pixel_texture: &Texture, x1: f32, y1: f32, x2: f32, y2: f32, color: &Color) {
  let px1 = x1 as i32;
  let py1 = y1 as i32;
  let px2 = x2 as i32;
  let py2 = y2 as i32;

  let dx = px2 - px1;
  let dy = py2 - py1;
  if dx.abs() > dy.abs() {
    let inc_x = if dx > 0 { 1 } else { -1 };
    let mut x = px1;
    while x != px1 + dx {
      let y = py1 + dy * (x - px1) / dx;
      pixel_texture.draw(ctx,
        DrawParams::new().position(Vec2::new(x as f32, y as f32)).scale(Vec2::new(2.0, 2.0)).color(*color),
      );
      x += inc_x;
    }
  } else {
    let inc_y = if dy > 0 { 1 } else { -1 };
    let mut y = py1;
    while y != py1 + dy {
      let x = px1 + dx * (y - py1) / dy;
      pixel_texture.draw(ctx,
        DrawParams::new().position(Vec2::new(x as f32, y as f32)).scale(Vec2::new(2.0, 2.0)).color(*color),
      );
      y += inc_y;
    }
  }
}

struct GameState {
  pixel_texture: Texture,
}

impl GameState {
  fn new(ctx: &mut Context) -> tetra::Result<Self> {
    Ok(Self {
      pixel_texture: Texture::from_rgba(ctx, 1, 1, &[255, 255, 255, 255])?,
    })
  }
}

impl State for GameState {
  fn update(&mut self, ctx: &mut Context) -> tetra::Result {
    if input::is_key_pressed(ctx, input::Key::Enter) &&
       (input::is_key_down(ctx, input::Key::LeftAlt) || input::is_key_down(ctx, input::Key::RightAlt)) {
         window::set_fullscreen(ctx, !window::is_fullscreen(ctx))?;
    }

    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
    let (screen_width, screen_height) = window::get_size(ctx);

    graphics::clear(ctx, Color::BLACK);
    draw_line(ctx, &self.pixel_texture, screen_width as f32 / 4.0, screen_height as f32 / 4.0, screen_width as f32 * 3.0 / 4.0, screen_height as f32 * 3.0 / 4.0, &Color::WHITE);
    draw_line(ctx, &self.pixel_texture, screen_width as f32 / 4.0, screen_height as f32 * 3.0 / 4.0, screen_width as f32 * 3.0 / 4.0, screen_height as f32 / 4.0, &Color::WHITE);

    Ok(())
  }
}

fn main() -> tetra::Result {
  // To hide the console in release builds only:
  #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

  ContextBuilder::new("Rust3DEngine", 1280, 960)
    .vsync(false)
    .show_mouse(true)
    .build()?
    .run(GameState::new)
}