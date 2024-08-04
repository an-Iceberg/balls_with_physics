#![allow(mixed_script_confusables)]

use egui_macroquad::{draw, egui::{epaint::Shadow, Align2, Slider, Visuals, Window}, ui};
use macroquad::{color::BLACK, rand, window::{clear_background, Conf}};
use macroquad::math::Vec2;
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
    // TODO: seed randomness using srand()
    let radius = rand::gen_range(4., 40.);
    let color = Color::from_rgba(
      rand::gen_range(64, 255),
      rand::gen_range(64, 255),
      rand::gen_range(64, 255),
      255
    );

    // Positions should not conflict with each other
    let position = Vec2::new(
      rand::gen_range(0., screen_width()),
      rand::gen_range(0., screen_height()),
    );

    return Ball { position, velocity: Vec2::ZERO, radius, color };
  }
}

fn add_new_ball(balls: &mut Vec<Ball>)
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
          &ball.position,
          ball.radius,
          &new_ball.position,
          new_ball.radius,
        )
      );

    // No overlaps, approve ball
    if first_overlap.is_none()
    {
      balls.push(new_ball);
      break;
    }
    else // Try new position for new ball
    {
      new_ball.position = Vec2::new(
        rand::gen_range(0., screen_width()),
        rand::gen_range(0., screen_height()),
      );
    }
  }
}

fn physics(balls: &mut [Ball], friction_mode: &FrictionMode)
{
  let mut collisions = vec![];

  // Pushing balls apart (static collision)
  for i in 0..balls.len()
  {
    for j in 0..balls.len()
    {
      // Don't check for collision with itself
      if i == j { continue; }

      if do_circles_overlap(&balls[i].position, balls[i].radius, &balls[j].position, balls[j].radius)
      {
        let distance = ((balls[i].position.x - balls[j].position.x).powi(2) + (balls[i].position.y - balls[j].position.y).powi(2)).sqrt();
        collisions.push((i, j));

        let overlap = 0.5 * (distance - balls[i].radius - balls[j].radius);
        // Displace current ball
        balls[i].position -= overlap * (balls[i].position - balls[j].position) / distance;
        // Displace target ball
        balls[j].position += overlap * (balls[i].position - balls[j].position) / distance;
      }
    }
  }

  // Physics simulation (dynamic collision)
  for (i, j) in collisions
  {
    // Show collision
    draw_line(balls[i].position.x, balls[i].position.y, balls[j].position.x, balls[j].position.y, 1., RED);

    // Formulas: https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional_collision_with_two_moving_objects
    let v_1 = balls[i].velocity;
    let v_2 = balls[j].velocity;
    let x_1 = balls[i].position;
    let x_2 = balls[j].position;
    let m_1 = balls[i].radius * 10.;
    let m_2 = balls[j].radius * 10.;
    let m_δ = m_1 + m_2;

    // Note: vec dot product != vec mul
    balls[i].velocity = v_1 - (2.*m_2 / m_δ) * (((v_1-v_2).dot(x_1-x_2)) / (x_1-x_2).length().powi(2)) * (x_1 - x_2);
    balls[j].velocity = v_2 - (2.*m_1 / m_δ) * (((v_2-v_1).dot(x_2-x_1)) / (x_2-x_1).length().powi(2)) * (x_2 - x_1);

    if matches!(friction_mode, FrictionMode::Collision)
    {
      balls[i].velocity *= 0.95;
      balls[j].velocity *= 0.95;
    }
  }

  // Updating ball positions & adjusting velocities
  for ball in balls
  {
    ball.position += ball.velocity * get_frame_time();

    if matches!(friction_mode, FrictionMode::Drag)
    { ball.velocity *= 0.99; }

    // Below a certain speed the ball will stand still
    if (ball.velocity.x.powi(2) + ball.velocity.y.powi(2)).abs() < 0.05
    { ball.velocity = Vec2::ZERO; }

    // Wrap around screen
    if ball.position.x <= 0.
    { ball.position.x = screen_width(); }
    else if ball.position.x > screen_width()
    { ball.position.x = 0. }

    if ball.position.y <= 0.
    { ball.position.y = screen_height(); }
    else if ball.position.y > screen_height()
    { ball.position.y = 0.; }
  }
}

#[allow(clippy::needless_return)]
pub fn is_point_in_circle(point: &Vec2, circle: &Vec2, radius: f32) -> bool
{ return (circle.x - point.x).powi(2) + (circle.y - point.y).powi(2) <= radius.powi(2); }

#[allow(clippy::needless_return)]
pub fn do_circles_overlap(circle_1: &Vec2, radius_1: f32, circle_2: &Vec2, radius_2: f32) -> bool
{ return ((circle_1.x - circle_2.x).powi(2) + (circle_1.y - circle_2.y).powi(2)) <= (radius_1 + radius_2).powi(2); }

#[derive(PartialEq, Eq)]
enum FrictionMode
{ Drag, Collision, None }

#[macroquad::main(window_config)]
async fn main()
{
  let mut balls = vec![];
  // Creating 100 balls, not overlapping
  balls.push(Ball::new());
  for _ in 0..99
  { add_new_ball(&mut balls); }

  let mut ball_thickness = 2.;
  let mut selected_ball = None;
  let mut selected_ball_vel: Option<usize> = None;
  let mut speed = 0.0;
  let mut friction_mode = FrictionMode::Drag;

  loop
  {
    clear_background(BLACK);

    // %%%%%%%%%%%%%%%%%%%% User input %%%%%%%%%%%%%%%%%%%%

    // Left click on ball makes a ball move around
    if is_mouse_button_pressed(MouseButton::Left)
    {
      // Left clicking again releases the ball
      if selected_ball.is_some()
      {
        selected_ball = None;
      }
      else
      {
        selected_ball_vel = None;
        let mouse = Vec2::from(mouse_position());
        selected_ball = None;
        for (i, _) in balls.iter().enumerate()
        {
          if is_point_in_circle(&mouse, &balls[i].position, balls[i].radius)
          {
            selected_ball = Some(i);
            // Picking the ball up sets its velocity to 0
            balls[i].velocity = Vec2::ZERO;
            break;
          }
        }
      }
    }

    // Right mouse button launches ball
    if is_mouse_button_pressed(MouseButton::Right)
    {
      if let Some(i) = selected_ball_vel
      {
        let mouse = Vec2::from(mouse_position());
        // balls[i].velocity = balls[i].position - mouse;
        balls[i].velocity = (balls[i].position - mouse).normalize() * speed;
        selected_ball_vel = None;
      }
      else
      {
        selected_ball = None;
        let mouse = Vec2::from(mouse_position());
        selected_ball_vel = None;
        for (i, _) in balls.iter().enumerate()
        {
          if is_point_in_circle(&mouse, &balls[i].position, balls[i].radius)
          {
            selected_ball_vel = Some(i);
            break;
          }
        }
      }
    }

    if mouse_wheel().1 != 0.
    {
      speed += mouse_wheel().1 * 100.;
      speed = clamp(speed, 0., 5000.);
    }

    // Cancel all inputs with esc
    if is_key_pressed(KeyCode::Escape)
    {
      selected_ball = None;
      selected_ball_vel = None;
    }

    // %%%%%%%%%%%%%%%%%%%% Update game logic %%%%%%%%%%%%%%%%%%%%

    if let Some(i) = selected_ball
    { balls[i].position = Vec2::from(mouse_position()); }

    physics(&mut balls, &friction_mode);

    // Draw balls
    balls.iter().for_each(|ball| draw_circle_lines(ball.position.x, ball.position.y, ball.radius, ball_thickness, ball.color));

    // Drawing an arrow in the direction of the ball launch
    if let Some(i) = selected_ball_vel
    {
      let mouse = Vec2::from(mouse_position());
      let (x_1, y_1) = (mouse.x, mouse.y);
      let (x_2, y_2) = (balls[i].position.x, balls[i].position.y);
      let l_1 = (mouse - balls[i].position).length();
      let l_2 = 20.;
      let θ: f32 = 0.436332;
      // Dynamic programming
      // meaning, we only compute stuff once and then store & reuse the results
      // b/c that's more efficient than to recompute them every time
      let x_δ = x_1 - x_2;
      let y_δ = y_1 - y_2;
      let l_δ = l_2 / l_1;
      let cos_θ = θ.cos();
      let sin_θ = θ.sin();
      // Formulas: https://math.stackexchange.com/questions/1314006/drawing-an-arrow
      let x_3 = x_2 + l_δ*(x_δ*cos_θ + y_δ*sin_θ);
      let y_3 = y_2 + l_δ*(y_δ*cos_θ - x_δ*sin_θ);
      let x_4 = x_2 + l_δ*(x_δ*cos_θ - y_δ*sin_θ);
      let y_4 = y_2 + l_δ*(y_δ*cos_θ + x_δ*sin_θ);

      draw_line(x_1, y_1, x_2, y_2, 1., GREEN);
      draw_line(x_2, y_2, x_3, y_3, 1., GREEN);
      draw_line(x_2, y_2, x_4, y_4, 1., GREEN);
    }

    // %%%%%%%%%%%%%%%%%%%% UI %%%%%%%%%%%%%%%%%%%%

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
        .anchor(Align2::RIGHT_TOP, egui_macroquad::egui::Vec2::new(-10., 10.))
        .fixed_size(egui_macroquad::egui::Vec2::new(170., 0.))
        .show(egui_context, |ui|
        {
          ui.label("Left click on a ball to move it around. Left click again to release it");
          ui.label("Right click on a ball to make a queue appear. Right click again to impart velocity on it.");
          ui.label("Mouse wheel increases/decreases speed.");
          ui.label("Esc cancels all actions.");
          ui.separator();
          ui.label("Line thickness");
          ui.add(Slider::new(&mut ball_thickness, 1_f32..=4.));
          ui.separator();
          ui.label("Speed");
          ui.add(Slider::new(&mut speed, 0_f32..=5000.));
          ui.separator();
          ui.label("How to apply friction");
          ui.radio_value(&mut friction_mode, FrictionMode::Drag, "Drag");
          ui.radio_value(&mut friction_mode, FrictionMode::Collision, "Collision");
          ui.radio_value(&mut friction_mode, FrictionMode::None, "None");
          ui.separator();
          if ui.button("Stop all motion").clicked()
          { balls.iter_mut().for_each(|ball| ball.velocity = Vec2::ZERO); }
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
