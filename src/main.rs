use std::fs::File;
use std::io::{BufRead, BufReader};

use tetra::{window, input, graphics, time};
use tetra::graphics::{Color, DrawParams, Texture};
use tetra::{Context, ContextBuilder, State};
use tetra::math::{Vec2, Vec4, Mat4};

const PI: f32 = 3.14159;
const PI2: f32 = 2.0 * PI;

const DBG_DRAW: bool = true;
const DBG_DRAW_FILL_TRIANGLE: bool = DBG_DRAW && true;
const DBG_DRAW_WIREFRAME: bool = DBG_DRAW && true;
const DBG_DRAW_NORMAL: bool = DBG_DRAW && false;

fn pixel_line(width: &mut Vec<(i32, i32)>, x1: i32, y1: i32, x2: i32, y2: i32) {
  let dx = (x2 - x1).abs();
  let sx : i32 = if x1 < x2 { 1 } else { -1 };
  let dy = -(y2 - y1).abs();
  let sy : i32 = if y1 < y2 { 1 } else { -1 };
  let mut err = dx + dy;
  let mut x = x1;
  let mut y = y1;

  loop {
    let index = (y - y1) as usize;
    if width.len() <= index {
      width.push((x, x));
    } else {
      let (x1, x2) = width[index];
      width[index] = (x1.min(x), x2.max(x));
    }
    if x == x2 && y == y2 { break; }
    let e = 2 * err;
    if e >= dy { err += dy; x += sx; }
    if e <= dx { err += dx; y += sy; }
  }
}

pub fn draw_fill_triangle(ctx: &mut Context, pixel_texture: &Texture, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: &Color) {
  if y1 <= y2 && y2 <= y3 {
    if y2 == y3 || y1 == y2 { // bottom flat or top flat
      let mut width : Vec<(i32, i32)> = Vec::new();
      if y2 == y3 {
        pixel_line(&mut width, x1, y1, x2, y2);
        pixel_line(&mut width, x1, y1, x3, y3);
      } else {
        pixel_line(&mut width, x1, y1, x3, y3);
        pixel_line(&mut width, x2, y2, x3, y3);
      }
      let mut y = y1;
      for (x1, x2) in &width {
        pixel_texture.draw(ctx,
          DrawParams::new().position(Vec2::new(*x1 as f32, y as f32)).scale(Vec2::new((*x2 - *x1 + 1) as f32, 1.0)).color(*color),
        );
        y += 1;
      }
    } else { // split in two triangle
      let x4 = x1 + ((((y2 - y1) as f32) / ((y3 - y1) as f32)) * ((x3 - x1) as f32)) as i32;
      let y4 = y2;
      draw_fill_triangle(ctx, pixel_texture, x1, y1, x2, y2, x4, y4, color);
      draw_fill_triangle(ctx, pixel_texture, x2, y2, x4, y4, x3, y3, color);
    }
  } else if y1 <= y3 && y3 <= y2 {
    draw_fill_triangle(ctx, pixel_texture, x1, y1, x3, y3, x2, y2, color);
  } else if y2 <= y1 && y1 <= y3 {
    draw_fill_triangle(ctx, pixel_texture, x2, y2, x1, y1, x3, y3, color);
  } else if y2 <= y3 && y3 <= y1 {
    draw_fill_triangle(ctx, pixel_texture, x2, y2, x3, y3, x1, y1, color);
  } else if y3 <= y1 && y1 <= y2 {
    draw_fill_triangle(ctx, pixel_texture, x3, y3, x1, y1, x2, y2, color);
  } else if y3 <= y2 && y2 <= y1 {
    draw_fill_triangle(ctx, pixel_texture, x3, y3, x2, y2, x1, y1, color);
  }
}

pub fn draw_line(ctx: &mut Context, pixel_texture: &Texture, mut x1: i32, mut y1: i32, x2: i32, y2: i32, color: &Color) {
  let dx = (x2 - x1).abs();
  let sx : i32 = if x1 < x2 { 1 } else { -1 };
  let dy = -(y2 - y1).abs();
  let sy : i32 = if y1 < y2 { 1 } else { -1 };
  let mut err = dx + dy;

  loop {
    pixel_texture.draw(ctx,
      DrawParams::new().position(Vec2::new(x1 as f32, y1 as f32)).scale(Vec2::new(1.0, 1.0)).color(*color),
    );
    if x1 == x2 && y1 == y2 { break; }
    let e2 = 2 * err;
    if e2 >= dy { err += dy; x1 += sx; }
    if e2 <= dx { err += dx; y1 += sy; }
  }
}

struct Triangle {
  vertices: [Vec4<f32>; 3], // clockwise points (x, y, z, w) of the triangle
}

struct Mesh {
  triangles: Vec<Triangle>,
  center: Vec4<f32>,
  rotation: [f32; 3],
  translation: Vec4<f32>,
  rotation_mat: Mat4<f32>,
}

struct GameState {
  pixel_texture: Texture,
  fov: f32,
  meshes: Vec<Mesh>,
  camera: Vec4<f32>,
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
              0.0);
    normal.normalize();
    normal
  }
}

impl Mesh {
  fn read_from_file(filename: &str) -> Self {
    let mut points: Vec<Vec4<f32>> = Vec::new();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut triangles: Vec<Triangle> = Vec::new();

    for line in reader.lines() {
      let line = line.unwrap();
      let split = line.split_whitespace().collect::<Vec<&str>>();
      if split.len() > 0 {
        if split[0] == "v" {
          points.push(Vec4::new(split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap(), split[3].parse::<f32>().unwrap(), 0.0));
        }
        else if split[0] == "f" {
          triangles.push(Triangle { vertices: [ points[split[1].parse::<usize>().unwrap() - 1], points[split[2].parse::<usize>().unwrap() - 1], points[split[3].parse::<usize>().unwrap() - 1] ], });
        }
      }
    }

    Self::new(triangles)
  }

  fn new(triangles: Vec<Triangle>) -> Self {
    let mut center = Vec4::new(0.0, 0.0, 0.0, 0.0);
    for triangle in &triangles {
      center += triangle.center();
    }
    center /= triangles.len() as f32;
    Self {
      triangles: triangles,
      center: center,
      rotation: [0.0, 0.0, 0.0],
      translation: Vec4::new(0.0, 0.0, 6.0, 0.0),
      rotation_mat: Mat4::identity(),
    }
  }

  fn update(&mut self) {
    self.rotation_mat = Mat4::identity();
    self.rotation_mat.rotate_x(self.rotation[0]);
    self.rotation_mat.rotate_y(self.rotation[1]);
    self.rotation_mat.rotate_z(self.rotation[2]);
  }

  fn draw(&self, ctx: &mut Context, pixel_texture: &Texture, camera: Vec4<f32>, transform: Mat4<f32>, screen_size: (f32, f32)) {
    let (screen_width, screen_height) = screen_size;
    for triangle in &self.triangles {
      let triangle_normal = triangle.normal();
      let triangle_normal_rotated = (triangle_normal * self.rotation_mat).normalized();
      let mut camera_to_triangle_vector = (((triangle.vertices[0] - self.center) * self.rotation_mat) + self.center + self.translation - camera).normalized();
      camera_to_triangle_vector.w = 0.0;
      let dot_product_triangle_camera = triangle_normal_rotated.dot(camera_to_triangle_vector);
      if dot_product_triangle_camera < 0.0 {
        if DBG_DRAW_FILL_TRIANGLE || DBG_DRAW_WIREFRAME {
          let mut p0 = (((triangle.vertices[0] - self.center) * self.rotation_mat) + self.center + self.translation) * transform;
          if p0.w != 0.0 { p0 /= p0.w; }
          let mut p1 = (((triangle.vertices[1] - self.center) * self.rotation_mat) + self.center + self.translation) * transform;
          if p1.w != 0.0 { p1 /= p1.w; }
          let mut p2 = (((triangle.vertices[2] - self.center) * self.rotation_mat) + self.center + self.translation) * transform;
          if p2.w != 0.0 { p2 /= p2.w; }

          p0.x = (p0.x + 1.0) * 0.5 * (screen_width as f32);
          p0.y = (p0.y + 1.0) * 0.5 * (screen_height as f32);
          p1.x = (p1.x + 1.0) * 0.5 * (screen_width as f32);
          p1.y = (p1.y + 1.0) * 0.5 * (screen_height as f32);
          p2.x = (p2.x + 1.0) * 0.5 * (screen_width as f32);
          p2.y = (p2.y + 1.0) * 0.5 * (screen_height as f32);

          if DBG_DRAW_FILL_TRIANGLE {
            draw_fill_triangle(ctx, &pixel_texture, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, p2.x as i32, p2.y as i32, &Color::RED);
          }
          if DBG_DRAW_WIREFRAME {
            draw_line(ctx, &pixel_texture, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, &Color::WHITE);
            draw_line(ctx, &pixel_texture, p1.x as i32, p1.y as i32, p2.x as i32, p2.y as i32, &Color::WHITE);
            draw_line(ctx, &pixel_texture, p0.x as i32, p0.y as i32, p2.x as i32, p2.y as i32, &Color::WHITE);
          }
        }
        if DBG_DRAW_NORMAL {
          let triangle_center = triangle.center();
          let mut p0 = (((triangle_center - self.center) * self.rotation_mat) + self.center + self.translation) * transform;
          if p0.w != 0.0 { p0 /= p0.w; }
          let mut p1 = (((triangle_center + triangle_normal - self.center) * self.rotation_mat) + self.center + self.translation) * transform;
          if p1.w != 0.0 { p1 /= p1.w; }

          p0.x = (p0.x + 1.0) * 0.5 * (screen_width as f32);
          p0.y = (p0.y + 1.0) * 0.5 * (screen_height as f32);
          p1.x = (p1.x + 1.0) * 0.5 * (screen_width as f32);
          p1.y = (p1.y + 1.0) * 0.5 * (screen_height as f32);

          draw_line(ctx, &pixel_texture, p0.x as i32, p0.y as i32, p1.x as i32, p1.y as i32, &Color::WHITE);
        }
      }
    }
  }
}

impl GameState {
  fn new(ctx: &mut Context) -> tetra::Result<Self> {
    Ok(Self {
      pixel_texture: Texture::from_rgba(ctx, 1, 1, &[255, 255, 255, 255])?,
      fov: (90.0 as f32).to_radians(),
      meshes: vec![
        Mesh::read_from_file("./assets/teapot.obj"),
      ],
      camera: Vec4::new(0.0, 0.0, 0.0, 0.0),
    })
  }
}

impl State for GameState {
  fn update(&mut self, ctx: &mut Context) -> tetra::Result {
    if input::is_key_pressed(ctx, input::Key::Enter) &&
       (input::is_key_down(ctx, input::Key::LeftAlt) || input::is_key_down(ctx, input::Key::RightAlt)) {
         window::set_fullscreen(ctx, !window::is_fullscreen(ctx))?;
    }

    let delta = time::get_delta_time(ctx).as_secs_f32();
    let rotation_delta = (delta * PI2 / 10.0) % PI2;
    for mesh in &mut self.meshes {
      mesh.rotation[0] += rotation_delta;
      mesh.rotation[1] += rotation_delta;
      mesh.rotation[2] += rotation_delta;
      mesh.update();
    }
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
    let (screen_width, screen_height) = window::get_size(ctx);
    let aspect_ratio = (screen_height as f32) / (screen_width as f32);
    let fov = 1.0 / (self.fov / 2.0).tan();
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
      mesh.draw(ctx, &self.pixel_texture, self.camera, projection, (screen_width as f32, screen_height as f32));
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