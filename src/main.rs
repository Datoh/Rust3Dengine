use tetra::{window, input, graphics};
use tetra::graphics::{Color, DrawParams, Texture};
use tetra::{Context, ContextBuilder, State};
use tetra::math::{Vec2, Vec4, Mat4};

const DBG_DRAW: bool = true;
const DBG_DRAW_WIREFRAME: bool = DBG_DRAW && true;
const DBG_DRAW_NORMAL: bool = DBG_DRAW && false;

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

struct Triangle {
  vertices: [Vec4<f32>; 3], // clockwise points (x, y, z, w) of the triangle
}

struct Mesh {
  triangles: Vec<Triangle>,
}

struct GameState {
  pixel_texture: Texture,
  fov: f32,
  meshes: Vec<Mesh>,
}

impl Triangle {
  pub fn center (&self) -> Vec4<f32> {
    Vec4::new((self.vertices[0].x + self.vertices[1].x + self.vertices[2].x) / 3.0,
              (self.vertices[0].y + self.vertices[1].y + self.vertices[2].y) / 3.0,
              (self.vertices[0].z + self.vertices[1].z + self.vertices[2].z) / 3.0,
              1.0)
  }
  pub fn normal(&self) -> Vec4<f32> {
    let a = self.vertices[1] - self.vertices[0];
    let b = self.vertices[2] - self.vertices[0];
    let mut normal = Vec4::new((a.y * b.z) - (a.z * b.y),
              (a.z * b.x) - (a.x * b.z),
              (a.x * b.y) - (a.y * b.x),
              1.0);
    normal.normalize();
    normal
  }
}

impl GameState {
  fn new(ctx: &mut Context) -> tetra::Result<Self> {
    Ok(Self {
      pixel_texture: Texture::from_rgba(ctx, 1, 1, &[255, 255, 255, 255])?,
      fov: 90.0,
      meshes: vec![
        Mesh { triangles: vec![
          Triangle { vertices: [ Vec4::new(0.0, 0.0, 3.0, 1.0), Vec4::new(0.0, 1.0, 3.0, 1.0), Vec4::new(1.0, 0.0, 3.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 1.0, 3.0, 1.0), Vec4::new(1.0, 1.0, 3.0, 1.0), Vec4::new(1.0, 0.0, 3.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 0.0, 4.0, 1.0), Vec4::new(1.0, 0.0, 4.0, 4.0), Vec4::new(0.0, 1.0, 4.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 1.0, 4.0, 1.0), Vec4::new(1.0, 0.0, 4.0, 4.0), Vec4::new(1.0, 1.0, 4.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 1.0, 3.0, 1.0), Vec4::new(0.0, 1.0, 4.0, 1.0), Vec4::new(1.0, 1.0, 4.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(1.0, 1.0, 4.0, 1.0), Vec4::new(1.0, 1.0, 3.0, 1.0), Vec4::new(0.0, 1.0, 3.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 0.0, 3.0, 1.0), Vec4::new(1.0, 0.0, 4.0, 1.0), Vec4::new(0.0, 0.0, 4.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(1.0, 0.0, 4.0, 1.0), Vec4::new(0.0, 0.0, 3.0, 1.0), Vec4::new(1.0, 0.0, 3.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 0.0, 3.0, 1.0), Vec4::new(0.0, 0.0, 4.0, 1.0), Vec4::new(0.0, 1.0, 3.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(0.0, 0.0, 4.0, 1.0), Vec4::new(0.0, 1.0, 4.0, 1.0), Vec4::new(0.0, 1.0, 3.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(1.0, 0.0, 3.0, 1.0), Vec4::new(1.0, 1.0, 3.0, 1.0), Vec4::new(1.0, 0.0, 4.0, 1.0) ], },
          Triangle { vertices: [ Vec4::new(1.0, 0.0, 4.0, 1.0), Vec4::new(1.0, 1.0, 3.0, 1.0), Vec4::new(1.0, 1.0, 4.0, 1.0) ], },
        ] },
      ]
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
    let aspect_ratio = (screen_height as f32) / (screen_width as f32);
    let fov = 1.0 / (self.fov / 2.0).to_radians().tan();
    let z_far: f32 = 1000.0;
    let z_near: f32 = 0.1;
    let q = z_far / (z_far - z_near);
    let projection = Mat4::new(
      aspect_ratio * fov, 0.0,         0.0, 0.0,
                     0.0, fov,         0.0, 0.0,
                     0.0, 0.0,           q, 1.0,
                     0.0, 0.0, -q * z_near, 0.0);

    graphics::clear(ctx, Color::BLACK);

    for mesh in &self.meshes {
      for triangle in &mesh.triangles {
        if DBG_DRAW_WIREFRAME {
          let mut p0 = triangle.vertices[0] * projection;
          if p0.w != 0.0 { p0 /= p0.w; }
          let mut p1 = triangle.vertices[1] * projection;
          if p1.w != 0.0 { p1 /= p1.w; }
          let mut p2 = triangle.vertices[2] * projection;
          if p2.w != 0.0 { p2 /= p2.w; }

          p0.x = (p0.x + 1.0) * 0.5 * (screen_width as f32);
          p0.y = (p0.y + 1.0) * 0.5 * (screen_height as f32);
          p1.x = (p1.x + 1.0) * 0.5 * (screen_width as f32);
          p1.y = (p1.y + 1.0) * 0.5 * (screen_height as f32);
          p2.x = (p2.x + 1.0) * 0.5 * (screen_width as f32);
          p2.y = (p2.y + 1.0) * 0.5 * (screen_height as f32);

          draw_line(ctx, &self.pixel_texture, p0.x, p0.y, p1.x, p1.y, &Color::WHITE);
          draw_line(ctx, &self.pixel_texture, p1.x, p1.y, p2.x, p2.y, &Color::WHITE);
          draw_line(ctx, &self.pixel_texture, p2.x, p2.y, p0.x, p0.y, &Color::WHITE);
        }
        if DBG_DRAW_NORMAL {
          let triangle_center = triangle.center();
          let mut p0 =  triangle_center* projection;
          let mut p1 = (triangle_center + triangle.normal()) * projection;
          if p0.w != 0.0 { p0 /= p0.w; }
          if p1.w != 0.0 { p1 /= p1.w; }

          p0.x = (p0.x + 1.0) * 0.5 * (screen_width as f32);
          p0.y = (p0.y + 1.0) * 0.5 * (screen_height as f32);
          p1.x = (p1.x + 1.0) * 0.5 * (screen_width as f32);
          p1.y = (p1.y + 1.0) * 0.5 * (screen_height as f32);
         
          draw_line(ctx, &self.pixel_texture, p0.x, p0.y, p1.x, p1.y, &Color::WHITE);
        }
      }
    }

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