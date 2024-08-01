use std::time::{SystemTime, UNIX_EPOCH};

use egui_macroquad::{draw, egui::{epaint::Shadow, Align2, Vec2, Visuals, Window}, ui};
use macroquad::{color::BLACK, rand, window::{clear_background, Conf}};
use macroquad::prelude::*;

pub(crate) const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
pub(crate) const AUTHORS: Option<&str> = option_env!("CARGO_PKG_AUTHORS");

#[allow(clippy::needless_return)]
pub fn window_config() -> Conf
{
  return Conf
  {
    window_title: "Balls with physics".to_string(),
    window_width: 1290,
    window_height: 720,
    fullscreen: false,
    window_resizable: false,
    sample_count: 4,
    ..Conf::default()
  };
}

struct Ball
{
  position: Vec2,
  velocity: Vec2,
  radius: f32,
  color: Color
}

impl Ball
{
  #[allow(clippy::needless_return)]
  pub fn new() -> Self
  {
    let radius = rand::gen_range(4., 40.);
    let color = Color::from_rgba(
      rand::gen_range(0, 255),
      rand::gen_range(0, 255),
      rand::gen_range(0, 255),
      255
    );

    let velocity = Vec2::new(
      rand::gen_range(-400., 400.),
      rand::gen_range(-400., 400.),
    );

    // Positions should not conflict with each other
    let position = Vec2::new(
      rand::gen_range(0., screen_width()),
      rand::gen_range(0., screen_height()),
    );

    return Ball { position, velocity: Vec2::ZERO, radius, color };
  }

  pub fn update(&mut self)
  {
    self.position += self.velocity * get_frame_time();

    // Reaching screen edge means loop around
    if self.position.x <= 0.
    {
      self.position.x = screen_width();
    }
    else if self.position.x >= screen_width()
    {
      self.position.x = 0.
    }

    if self.position.y <= 0.
    {
      self.position.y = screen_height();
    }
    else if self.position.y >= screen_height()
    {
      self.position.y = 0.;
    }
  }
}

#[macroquad::main(window_config)]
async fn main()
{
  let mut balls = vec![];
  // Creating 100 balls
  balls.push(Ball::new());
  (0..99)
    .for_each(|i|
    {
      // Create new ball
      let mut new_ball = Ball::new();
      // Check if it is inside any other ball(s)
      loop
      {
        // Find the first ball it overlaps with
        let first_overlap = balls.iter()
          .position(|ball|
            do_circles_overlap(
              ball.position,
              ball.radius,
              new_ball.position,
              new_ball.radius,
            )
          );

        // No overlaps, approve ball
        if first_overlap.is_none()
        {
          balls.push(new_ball);
          break;
        }
        // Try new position for new ball
        else
        {
          new_ball.position = Vec2::new(
            rand::gen_range(0., screen_width()),
            rand::gen_range(0., screen_height()),
          );
        }
      }
    });

  loop
  {
    clear_background(BLACK);

    // Update game logic
    balls.iter_mut()
      .for_each(|ball| ball.update());

    // Draw balls
    balls.iter()
      .for_each(|ball| draw_circle(ball.position.x, ball.position.y, ball.radius, ball.color));

    // TODO: ball collision physics
    // TODO: allow user to apply impulse to balls
    // TODO: optional friction

    // Draw UI
    ui(|egui_context|
    {
      egui_context.set_visuals(Visuals
      {
        window_shadow: Shadow::NONE,
        ..Default::default()
      });

      Window::new("")
        .movable(false)
        .resizable(false)
        .anchor(Align2::RIGHT_TOP, Vec2::new(-10., 10.))
        .fixed_size(Vec2::new(150., 0.))
        .show(egui_context, |ui|
        {
          ui.label("Hello world");

          ui.separator();

          ui.label(format!("# of balls: {}", balls.len()));

          ui.separator();

          ui.label(format!("fps: {}", get_fps()));

          ui.separator();

          // --- CREDITS (!important) ---
          ui.horizontal(|ui|
          {
            ui.label(format!("v{}", VERSION.unwrap_or("unknown")));
            ui.separator();
            ui.label("Made by");
            ui.hyperlink_to(AUTHORS.unwrap_or("Sandra").to_string(), "https://github.com/an-Iceberg");
          });
        });

    });

    draw();

    next_frame().await;
  }
}

#[allow(clippy::needless_return)]
pub fn is_point_in_circle(
  point: Vec2, circle: Vec2, radius: f32
) -> bool
{
  return (circle.x - point.x).powi(2) + (circle.y - point.y).powi(2) <= radius.powi(2);
}

#[allow(clippy::needless_return)]
pub fn do_circles_overlap(
  circle_1: Vec2, radius_1: f32,
  circle_2: Vec2, radius_2: f32,
) -> bool
{
  return ((circle_1.x - circle_2.x).powi(2) + (circle_1.y - circle_2.y).powi(2)) <= (radius_1 + radius_2).powi(2);
}
