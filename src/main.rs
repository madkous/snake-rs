use nalgebra as na;
use ggez::*;
use ggez::graphics::{Canvas,Drawable,DrawParam};
use ggez::conf::{NumSamples};
use ggez::input::keyboard::{KeyCode, KeyMods};
use rand;
use specs::*;
use specs_derive::{Component};
// use std::ops::{Add, AddAssign};
// use std::{path,env};

const GRID_SIZE: Position = Position{ x: 30, y: 30 };
// time between updates in seconds
const UPDATE_DURATION: f64 = 0.10;
const CELL_SIZE: f32 = 20.0;

pub type Point32 = na::Point2<f32>;
#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction { Up, Down, Left, Right, }

impl Direction {
	fn opposite(&self) -> Direction {
		match self {
			Self::Up => Self::Down,
			Self::Down => Self::Up,
			Self::Left => Self::Right,
			Self::Right => Self::Left,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Position {
	x: i16,
	y: i16,
}

impl Position {
	fn new(x: i16, y: i16) -> Self{
		Position{ x, y, }
	}

	fn out_of_bounds(self) -> bool {
		self.x < 0
			|| self.y < 0
			|| self.x > GRID_SIZE.x
			|| self.y > GRID_SIZE.y
	}
}

////////////////
// Components //
////////////////
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
struct Snake {
	length: i16,
	segments: Vec<Position>,
	direction: Direction,
}

impl Snake {
	fn advance(&mut self) {
		let Position{x, y} = self.segments[0];
		self.segments.insert(0, match self.direction {
			Direction::Up => Position::new(x, y - 1),
			Direction::Down => Position::new(x, y + 1),
			Direction::Left => Position::new(x - 1, y),
			Direction::Right => Position::new(x + 1, y),
		});
		self.segments.pop();
		assert_eq!(self.length as usize, self.segments.len());
	}

	fn grow(&mut self) {
		let Position{x, y} = self.segments[0];
		self.segments.insert(0, match self.direction {
			Direction::Up => Position::new(x, y - 1),
			Direction::Down => Position::new(x, y + 1),
			Direction::Left => Position::new(x - 1, y),
			Direction::Right => Position::new(x + 1, y),
		});
		self.length += 1;
		assert_eq!(self.length as usize, self.segments.len());
	}

	fn get_eat(&self) -> Position {
		let Position{x, y} = self.segments[0];
		match self.direction {
			Direction::Up => Position::new(x, y - 1),
			Direction::Down => Position::new(x, y + 1),
			Direction::Left => Position::new(x - 1, y),
			Direction::Right => Position::new(x + 1, y),
		}
	}

	fn out_of_bounds(&self) -> bool {
		self.segments[0].out_of_bounds()
	}

	fn self_intersect(&self) -> bool {
		if let Some((head, segments)) = self.segments.split_first() {
			for seg in segments {
				if head == seg {
					return true;
				}
			}
		}
		false
	}
}

impl Default for Snake {
	fn default() -> Self {
		Snake {
			length: 1,
			segments: vec!(Position::new(10,10)),
			direction: Direction::Right,
		}
	}
}

#[derive(Copy, Clone, Debug, Component)]
#[storage(VecStorage)]
struct Food(Position);

impl Food {
	fn new() -> Self {
		let x = rand::random::<i16>() % (GRID_SIZE.x / 2) + (GRID_SIZE.x / 2);
		let y = rand::random::<i16>() % (GRID_SIZE.y / 2) + (GRID_SIZE.y / 2);
		Food(Position{ x, y, })
	}
}

#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
struct Color(graphics::Color);

macro_rules! add_color {
	( $col:literal, $name:ident ) => {
		fn $name() -> Self {
			Color(graphics::Color::from_rgb_u32($col))
		}
	};
}

impl Color {
	add_color!(0x000000, black);
	// add_color!(0xce281f, red);
	add_color!(0x00853e, green);
	// add_color!(0xe8bf04, yellow);
	add_color!(0x009ddc, blue);
	// add_color!(0x98005d, magenta);
	// add_color!(0x2cba96, cyan);
	add_color!(0xc3c2c2, white);
	// add_color!(0x454444, brblack);
	// add_color!(0xee534b, brred);
	// add_color!(0x2f8557, brgreen);
	// add_color!(0xffde47, bryellow);
	// add_color!(0x5cb7dc, brblue);
	// add_color!(0x983f75, brmagenta);
	// add_color!(0x97cec0, brcyan);
	add_color!(0xe3e3e3, brwhite);
}

impl Default for Color {
	fn default() -> Self {
		Color::brwhite()
	}
}

/////////////
// Systems //
/////////////

struct SnakeSystem;

impl<'a> System<'a> for SnakeSystem {
	type SystemData = (Read<'a, world::EntitiesRes>,
					   WriteStorage<'a, Snake>,
					   WriteStorage<'a, Food>);

	fn run(&mut self, (entities, mut snake, mut food): Self::SystemData) {
		for s in (&mut snake).join() {
			let mut moved = false;
			let pos = s.get_eat();
			for (e, f) in (&entities, &food).join() {
				if f.0 == pos {
					s.grow();
					entities.delete(e).expect("You ate a ghost!");
					let new = entities.create();
					food.insert(new, Food::new()).unwrap(); // TODO make better
					moved = true;
					break;
				}
			} if !moved {
				s.advance();
			}
		}
	}
}

struct BoundSystem;

impl<'a> System<'a> for BoundSystem {
	type SystemData = (Write<'a, Playing>,
					   ReadStorage<'a, Snake>);

	fn run(&mut self, (mut playing, mut snake): Self::SystemData) {
		for s in (&mut snake).join() {
			// println!("self_intersect = {}", s.self_intersect());
			// println!("out_of_bounds = {}", s.out_of_bounds());
			*playing = Playing(!(s.self_intersect() || s.out_of_bounds()));
		}
	}
}

struct InputSystem;

impl<'a> System<'a> for InputSystem {
	type SystemData = (Read<'a, Input>,
					   WriteStorage<'a, Snake>);

	fn run(&mut self, (input, mut snake): Self::SystemData) {
		for s in (&mut snake).join() {
			if s.direction != input.direction.opposite() {
				s.direction = input.direction;
			}
		}
	}
}

struct RenderSystem<'c> {
	ctx: &'c mut Context,
}

impl<'c> RenderSystem<'c> {
	pub fn new(ctx: &'c mut Context) -> RenderSystem<'c> {
		RenderSystem { ctx }
	}
}

impl<'a, 'c> System<'a> for RenderSystem<'c> {
	type SystemData = (ReadExpect<'a, Canvas>,
					   ReadStorage<'a, Snake>,
					   ReadStorage<'a, Food>);

	fn run(&mut self, (canvas, snake, food): Self::SystemData) {
		graphics::set_canvas(self.ctx, Some(&canvas));
		graphics::clear(self.ctx, Color::black().0);
		for f in (&food).join() {
			if let Ok(mesh) = graphics::Mesh::new_rectangle(
				self.ctx,
				graphics::DrawMode::fill(),
				graphics::Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE),
				Color::blue().0,
			) {
				let pos = Point32::new(f.0.x as f32 * CELL_SIZE,
									   f.0.y as f32 * CELL_SIZE);
				graphics::draw(self.ctx, &mesh, (pos,))
					.expect("Could not draw mesh");
			}
		}
		for s in (&snake).join() {
			for seg in &s.segments {
				if let Ok(mesh) = graphics::Mesh::new_rectangle(
					self.ctx,
					graphics::DrawMode::fill(),
					graphics::Rect::new(0.0, 0.0, CELL_SIZE, CELL_SIZE),
					Color::green().0,
					) {
					let pos = Point32::new(seg.x as f32 * CELL_SIZE,
										   seg.y as f32 * CELL_SIZE);
					graphics::draw(self.ctx, &mesh, (pos,))
						.expect("Could not draw mesh");
				}
			}
		}
		graphics::set_canvas(self.ctx, None);
		canvas.draw(self.ctx, DrawParam::default()
					.dest(Point32::new(5.0, 5.0)))
			.expect("Could not draw game canvas");
		// graphics::draw(self.ctx, &canvas, (Point32::new(5.0, 5.0),))
		// 				.expect("Could not draw mesh");
	}
}

///////////////
// Resources //
///////////////
struct Input {
	direction: Direction,
	pause: bool,
}

impl Default for Input {
	fn default () -> Self {
		Input {
			direction: Direction::Right,
			pause: false,
		}
	}
}

struct Playing(bool);

impl Default for Playing {
	fn default () -> Self {
		Playing(true)
	}
}

///////////////
// MainState //
///////////////

struct MainState<'a, 'b> {
	frames: usize,
	dt: f64,
	world: World,
	dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> MainState<'a, 'b> {
	fn new(ctx: &mut Context) -> GameResult<MainState<'a, 'b>> {
		let mut world = World::new();
		world.register::<Snake>();
		world.register::<Food>();
		world.register::<Color>();

		let dispatcher = DispatcherBuilder::new()
			.with(SnakeSystem, "snake_system", &[])
			.with(InputSystem, "input_system", &[])
			.with(BoundSystem, "bound_system", &[])
			.build();

		world.create_entity()
			.with(Snake::default())
			.build();

		world.create_entity()
			.with(Food::new())
			.build();

		world.insert(Input::default());
		world.insert(Playing::default());
		world.insert(Canvas::new(ctx,
								 GRID_SIZE.x as u16 * CELL_SIZE as u16,
								 GRID_SIZE.y as u16 * CELL_SIZE as u16,
								 NumSamples::One,
								 ).expect("error creating canvas"));

		println!("Finishing setup");
		Ok(MainState {
			frames: 0,
			dt: 0.0,
			world,
			dispatcher,
		})
	}
}

impl<'a, 'b> event::EventHandler for MainState<'a, 'b> {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
		let delta = timer::delta(ctx).as_secs_f64();
		if self.dt + delta >= UPDATE_DURATION {
			self.dt -= UPDATE_DURATION;
			if !self.world.read_resource::<Input>().pause {
				self.dispatcher.dispatch(&mut self.world);
				self.world.maintain();
			}

			self.frames += 1;
			if !self.world.read_resource::<Playing>().0 {
				println!("Lost after {} frames", self.frames);
				// println!("{:?}", self.world.read_resource::<Playing>().0);
				ggez::event::quit(ctx);
			}
		} else {
			self.dt += delta;
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, Color::white().0);
		{
			let mut rs = RenderSystem::new(ctx);
			rs.run_now(&mut self.world);
		}

		graphics::present(ctx)?;
		Ok(())
	}

	fn key_down_event(
		&mut self,
		ctx: &mut Context,
		keycode: KeyCode,
		_keymods: KeyMods,
		_repeat: bool
	) {
		let mut input = self.world.write_resource::<Input>();
		match keycode {
			KeyCode::Up => input.direction = Direction::Up,
			KeyCode::Down => input.direction = Direction::Down,
			KeyCode::Left => input.direction = Direction::Left,
			KeyCode::Right => input.direction = Direction::Right,
			KeyCode::Space => input.pause = !input.pause,
			KeyCode::Escape => ggez::event::quit(ctx),
			_ => (),
		};
	}
}

fn main() {
	// let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
	// 	let mut path = path::PathBuf::from(manifest_dir);
	// 	path.push("resources");
	// 	path
	// } else {
	// 	path::PathBuf::from("./resources")
	// };
	// println!("Resource dir: {:?}", resource_dir);

	let (mut ctx, mut event_loop) = ContextBuilder::new("snake", "madkous")
		.window_setup(conf::WindowSetup::default().title("Swim Swim Hungry"))
		.window_mode(conf::WindowMode::default().dimensions(610.0, 610.0))
		// .add_resource_path(&resource_dir)
		.build()
		.expect("AIEEE! Could not create ggez context!");

	match MainState::new(&mut ctx) {
		Ok(mut state) =>
			if let Err(e) = event::run(&mut ctx, &mut event_loop, &mut state) {
				println!("Oops: {}", e);
			} else {
				println!("Phew! We made it!");
			},
		Err(e) => println!("Could not create game state: {}", e),
	}
}

