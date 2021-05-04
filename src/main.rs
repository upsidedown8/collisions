use ggez::{
    conf::{WindowMode, WindowSetup},
    graphics::{self, Color},
};
use ggez::{
    event::{self, EventHandler},
    graphics::Mesh,
};
use ggez::{Context, ContextBuilder, GameResult};
use rand::{prelude::SliceRandom, Rng};

type Vector = ggez::mint::Vector2<f32>;
type Point = ggez::mint::Point2<f32>;

// dimensions
const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;

// restitution coefficient
const RESTITUTION: f32 = 1.0;

// acceleration
const ACCELERATION: Vector = Vector { x: -1.0, y: 2.0 };
const RESISTANCE: Vector = Vector { x: 0.0, y: 0.0 };

// how many particles?
const NUM_PARTICLES: usize = 20;

fn main() -> GameResult {
    let (mut ctx, mut event_loop) = ContextBuilder::new("collisions", "Tom Thorogood")
        .window_mode(WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .window_setup(WindowSetup::default().title("Collisions"))
        .build()?;
    let mut my_game = GameState::new(&mut ctx);
    event::run(&mut ctx, &mut event_loop, &mut my_game)
}
struct GameState {
    particles: Vec<Particle>,
}

impl GameState {
    pub fn new(_ctx: &mut Context) -> GameState {
        let colors = vec![
            Color::from_rgb(170, 216, 211),
            Color::from_rgb(50, 175, 230),
            Color::from_rgb(0, 173, 181),
            Color::from_rgb(10, 17, 200),
            Color::from_rgb(150, 150, 20),
            Color::from_rgb(0, 90, 45),
            Color::from_rgb(200, 100, 50),
        ];

        let mut particles = Vec::new();

        for _ in 0..NUM_PARTICLES {
            let rad = rand::thread_rng().gen_range(7.5..12.5);
            let mass = rand::thread_rng().gen_range(1.0..1.25);
            let color = colors
                .choose(&mut rand::thread_rng())
                .expect("Some colors in the vec");
            let x = rand::thread_rng().gen_range(rad..SCREEN_WIDTH - rad);
            let y = rand::thread_rng().gen_range(rad..SCREEN_HEIGHT - rad);

            particles.push(Particle::new(
                Point { x, y },
                Vector {
                    x: rand::thread_rng().gen_range(-80.0..80.0),
                    y: rand::thread_rng().gen_range(-80.0..80.0),
                },
                rad,
                mass,
                *color,
            ));
        }

        // Load/create resources here: images, fonts, sounds, etc.
        GameState { particles }
    }

    fn handle_collisions(&mut self) {
        let num_particles = self.particles.len();

        // collisions
        for i in 0..num_particles - 1 {
            for j in i + 1..num_particles {
                if self.particles[i].is_colliding(&self.particles[j]) {
                    println!(
                        "collision at distance: {}",
                        self.particles[i].distance(&self.particles[j])
                    );

                    // u1
                    let u1_x = self.particles[i].vel.x;
                    let u1_y = self.particles[i].vel.y;

                    // u2
                    let u2_x = self.particles[j].vel.x;
                    let u2_y = self.particles[j].vel.y;

                    // m2 / m1
                    let m2_div_m1 = self.particles[j].mass / self.particles[i].mass;

                    // v1
                    self.particles[i].vel.x = ((1.0 - RESTITUTION) / 2.0 * u1_x)
                        + ((m2_div_m1 + RESTITUTION) / 2.0 * u2_x);
                    self.particles[i].vel.y = ((1.0 - RESTITUTION) / 2.0 * u1_y)
                        + ((m2_div_m1 + RESTITUTION) / 2.0 * u2_y);

                    // v2
                    self.particles[j].vel.x = ((1.0 + RESTITUTION) / 2.0 * u1_x)
                        + ((m2_div_m1 - RESTITUTION) / 2.0 * u2_x);
                    self.particles[j].vel.y = ((1.0 + RESTITUTION) / 2.0 * u1_y)
                        + ((m2_div_m1 - RESTITUTION) / 2.0 * u2_y);

                    let magnitude_1 = self.particles[i].vel_magnitude();
                    let magnitude_2 = self.particles[j].vel_magnitude();

                    let a = (self.particles[i].vel.x / magnitude_1).abs();
                    let b = (self.particles[i].vel.y / magnitude_1).abs();
                    let c = (self.particles[j].vel.x / magnitude_2).abs();
                    let d = (self.particles[j].vel.y / magnitude_2).abs();

                    let new_color = Color::from_rgb(
                        ((a * b) * 256.0) as u8,
                        ((c * d) * 256.0) as u8,
                        ((d * a) * 256.0) as u8,
                    );

                    self.particles[i].color = new_color;
                    self.particles[j].color = new_color;
                }
            }
        }
    }
    fn handle_movement(&mut self, time_elapsed: f32) {
        for particle in &mut self.particles {
            particle.update(time_elapsed);
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let time_elapsed = ggez::timer::delta(ctx).as_secs_f32();

        self.handle_collisions();
        self.handle_movement(time_elapsed);

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        let params = graphics::DrawParam::default();

        for particle in &self.particles {
            let mesh = particle.mesh(ctx)?;

            graphics::draw(ctx, &mesh, params)?;

            let line = graphics::Mesh::new_line(
                ctx,
                &[
                    particle.pos,
                    Point {
                        x: particle.pos.x + particle.vel.x,
                        y: particle.pos.y + particle.vel.y,
                    },
                ],
                2.0,
                particle.color,
            )?;

            graphics::draw(ctx, &line, params)?;
        }

        graphics::present(ctx)
    }
}

struct Particle {
    pub rad: f32,
    pub pos: Point,
    pub mass: f32,
    pub vel: Vector,
    color: Color,
}

impl Particle {
    pub fn new(pos: Point, vel: Vector, rad: f32, mass: f32, color: Color) -> Particle {
        Particle {
            pos,
            vel,
            rad,
            mass,
            color,
        }
    }
    pub fn mesh(&self, ctx: &mut Context) -> GameResult<Mesh> {
        graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.pos,
            self.rad,
            0.05,
            self.color,
        )
    }
    pub fn update(&mut self, time_elapsed: f32) {
        // bound checks
        // left/right
        if (self.pos.x - self.rad) < 0.0 {
            self.vel.x = self.vel.x.abs() * RESTITUTION;
        } else if (self.pos.x + self.rad) > SCREEN_WIDTH {
            self.vel.x = self.vel.x.abs() * -RESTITUTION;
        }

        // top/bottom
        if (self.pos.y - self.rad) < 0.0 {
            self.vel.y = self.vel.y.abs() * RESTITUTION;
        } else if (self.pos.y + self.rad) > SCREEN_HEIGHT {
            self.vel.y = self.vel.y.abs() * -RESTITUTION;
        }

        self.pos.x += self.vel.x * time_elapsed;
        self.pos.y += self.vel.y * time_elapsed;

        // resistance increases with vel squared
        let resistance_x = self.vel.x * self.vel.x * RESISTANCE.x;
        let resistance_y = self.vel.y * self.vel.y * RESISTANCE.y;

        self.vel.x += (ACCELERATION.x - resistance_x) * time_elapsed;
        self.vel.y += (ACCELERATION.y - resistance_y) * time_elapsed;
    }
    pub fn is_colliding(&self, other: &Particle) -> bool {
        self.distance(other) - (self.rad + other.rad) <= 0.5
    }
    pub fn distance(&self, other: &Particle) -> f32 {
        let dx = self.pos.x - other.pos.x;
        let dy = self.pos.y - other.pos.y;
        (dx * dx + dy * dy).sqrt()
    }
    pub fn vel_magnitude(&self) -> f32 {
        (self.vel.x * self.vel.x + self.vel.y * self.vel.y).sqrt()
    }
}
