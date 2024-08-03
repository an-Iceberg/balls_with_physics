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
    // Try new position for new ball
    else
    {
      new_ball.position = Vec2::new(
        rand::gen_range(0., screen_width()),
        rand::gen_range(0., screen_height()),
      );
    }
  }
}

fn physics(balls: &mut [Ball])
{
  let mut collisions = vec![];

  // Pushing balls apart (static collision)
  for i in 0..balls.len()
  {
    for j in 0..balls.len()
    {
      if i == j { continue; }

      if do_circles_overlap(&balls[i].position, balls[i].radius, &balls[j].position, balls[j].radius)
      {
        let distance = ((balls[i].position.x - balls[j].position.x).powi(2) + (balls[i].position.y - balls[j].position.y).powi(2)).sqrt();
        collisions.push((i, j, distance));

        let overlap = 0.5 * (distance - balls[i].radius - balls[j].radius);
        // Displace current ball
        balls[i].position -= overlap * (balls[i].position - balls[j].position) / distance;
        // Displace target ball
        balls[j].position += overlap * (balls[i].position - balls[j].position) / distance;
      }
    }
  }

  // Physics simulation (dynamic collision)
  collisions.iter()
    .for_each(|(i, j, distance)|
    {
      // Show collision
      draw_line(balls[*i].position.x, balls[*i].position.y, balls[*j].position.x, balls[*j].position.y, 1., RED);

      // Bug: something imparts tremendous amounts of momentum in a collision
      // Formulas: https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional_collision_with_two_moving_objects
      let normal = vec2(
        (balls[*j].position.x - balls[*i].position.x) / distance,
        (balls[*j].position.y - balls[*i].position.y) / distance,
      );
      let k = balls[*i].velocity - balls[*j].velocity;
      let p = 2.*(normal.x*k.x + normal.y*k.y) / (balls[*i].radius*10. + balls[*j].radius*10.);
      balls[*i].velocity -= p*balls[*j].radius*10.*normal;
      balls[*j].velocity += p*balls[*i].radius*10.*normal;
    });

  balls.iter_mut()
    .for_each(|ball|
    {
      ball.position += ball.velocity * get_frame_time();

      // Todo: friction/loss of energy
      // Apply drag
      ball.velocity *= 0.99;

      // Below a certain speed the ball will stand still
      if (ball.velocity.x.powi(2) + ball.velocity.y.powi(2)).abs() < 0.05
      { ball.velocity = Vec2::ZERO; }

      // Todo: make this work better with collision, otherwise the balls gain insane velocity
      // Wrap around screen
      if ball.position.x < 0.
      { ball.position.x = screen_width(); }
      else if ball.position.x >= screen_width()
      { ball.position.x = 0. }

      if ball.position.y < 0.
      { ball.position.y = screen_height(); }
      else if ball.position.y >= screen_height()
      { ball.position.y = 0.; }
    });
}

#[allow(clippy::needless_return)]
pub fn is_point_in_circle(point: &Vec2, circle: &Vec2, radius: f32) -> bool
{ return (circle.x - point.x).powi(2) + (circle.y - point.y).powi(2) <= radius.powi(2); }

#[allow(clippy::needless_return)]
pub fn do_circles_overlap(circle_1: &Vec2, radius_1: f32, circle_2: &Vec2, radius_2: f32) -> bool
{ return ((circle_1.x - circle_2.x).powi(2) + (circle_1.y - circle_2.y).powi(2)) <= (radius_1 + radius_2).powi(2); }

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

  loop
  {
    clear_background(BLACK);

    // User input
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
        balls[i].velocity = balls[i].position - mouse;
        selected_ball_vel = None;
      }
      else
      {
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

    // Cancel with esc
    if is_key_pressed(KeyCode::Escape)
    {
      selected_ball = None;
      selected_ball_vel = None;
    }

    // Update game logic
    if let Some(i) = selected_ball
    { balls[i].position = Vec2::from(mouse_position()); }

    physics(&mut balls);

    // Draw balls
    balls.iter()
      .for_each(|ball| draw_circle_lines(ball.position.x, ball.position.y, ball.radius, ball_thickness, ball.color));

    // Drawing the «queue»
    if let Some(i) = selected_ball_vel
    {
      let mouse = Vec2::from(mouse_position());
      draw_line(balls[i].position.x, balls[i].position.y, mouse.x, mouse.y, 1., GREEN);
    }

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
        .anchor(Align2::RIGHT_TOP, egui_macroquad::egui::Vec2::new(-10., 10.))
        .fixed_size(egui_macroquad::egui::Vec2::new(150., 0.))
        .show(egui_context, |ui|
        {
          ui.label("Left click on a ball to move it around. Left click again to release it");
          ui.label("Right click on a ball to make a queue appear. Right click again to impart velocity on it.");
          ui.label("Esc cancels all actions.");
          ui.separator();
          ui.label("Line thickness");
          ui.add(Slider::new(&mut ball_thickness, 1_f32..=4.));
          ui.separator();
          ui.label("Friction mode");
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
