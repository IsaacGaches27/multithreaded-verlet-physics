/*mod solver;
mod physics;

use solver::Solver;
use physics::Grid;
use coffee::{graphics::{ Batch, Color, Frame, Image, Point, Rectangle, Sprite, Window, WindowSettings}, load::{Join, Task},Game, Result, Timer};
use rayon::prelude::*;
use scoped_threadpool::Pool;

use std::time::SystemTime;
use coffee::graphics::Vector;

const WIDTH:usize = 2000;
const HEIGHT:usize = 1000;
const RADIUS:f32 = 2.2;
const STEPS:u32 = 12;
const GRAVITY:f32 = 0.3;
fn main() -> Result<()> {
    Main::run(WindowSettings {
        title: String::from("Particles - Coffee"),
        size: (WIDTH as u32, HEIGHT as u32),
        resizable: false,
        fullscreen: false,
        maximized: false,
    })
}
struct Main {
    solver: Solver,
    batch: Batch,
    thread_pool: Pool,
}
impl Main{
    fn initialise_solver()->Task<Solver>{
        Task::succeed(Solver::new)
    }
    fn load_sprite() -> Task<Image>{
        Image::load("resources/Circle.png")
    }
    fn generate_thread_pool()->Task<Pool> {
        Task::succeed(||Pool::new(num_cpus::get() as u32))
    }
}
impl Game for Main{
    type Input = ();
    type LoadingScreen = ();
    const TICKS_PER_SECOND: u16 = 60;
    fn load(_window: &Window) -> Task<Main>{
        (
            Task::stage("Initialising", Self::initialise_solver(), ),
            Task::stage("Loading Assets", Self::load_sprite()),
            Task::stage("Multithreading", Self::generate_thread_pool()),
        )
            .join()
            .map(|(solver, sprite,thread_pool)| Main {
                solver,
                batch: Batch::new(sprite),
                thread_pool
            })
    }
    fn update(&mut self, _window: &Window) {
        let now = SystemTime::now();
        let dt = 1.0/STEPS as f32;
        for _s in 0..STEPS{
            unsafe {
                self.solver.update_positions(dt);
                self.solver.set_cells();
                self.solver.solve_collisions_threaded(&mut self.thread_pool);
            }
        }
        println!("fps: {}",1000000./now.elapsed().unwrap().as_micros() as f32);
        if self.solver.object_count < 100000 {
            for i in 0..40{
                for j in 0..3{
                    self.solver.new_object(100. + j as f32 *6.0, 50. + i as f32*6.0);
                }
            }
        }

    }
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
        let scale = RADIUS/28.0;
        unsafe {
            let sprites = (*self.solver.objects.get()).par_iter().map(|object| {
                Sprite {
                    source: Rectangle {
                        x: 0,
                        y: 0,
                        width: 56,
                        height: 56,
                    },
                    position: Point::new(object.position.x - RADIUS, object.position.y - RADIUS),
                    scale: (scale, scale),
                }
            });
            self.batch.clear();
            self.batch.par_extend(sprites);
            self.batch.draw(&mut frame.as_target());
        }
    }
}*/

mod solver;
mod physics;

use solver::Solver;
use physics::Grid;
use coffee::{graphics::{ Batch, Color, Frame, Image, Point, Rectangle, Sprite, Window, WindowSettings}, load::{Join, Task},Game, Result, Timer};
use rayon::prelude::*;
use scoped_threadpool::Pool;

use std::time::SystemTime;
use coffee::graphics::Vector;

const WIDTH:usize = 1600;
const HEIGHT:usize = 1000;
const RADIUS:f32 = 2.2;
const STEPS:u32 = 8;
const GRAVITY:f32 = 0.08;
const OBJECTS_PER_CELL: usize = 4;

fn main() -> Result<()> {
    Main::run(WindowSettings {
        title: String::from("Particles - Coffee"),
        size: (WIDTH as u32, HEIGHT as u32),
        resizable: false,
        fullscreen: false,
        maximized: false,
    })
}
struct Main {
    solver: Grid,
    batch: Batch,
    thread_pool: Pool,
}
impl Main{
    fn initialise_solver()->Task<Grid>{
        Task::succeed(Grid::new)
    }
    fn load_sprite() -> Task<Image>{
        Image::load("resources/Circle.png")
    }
    fn generate_thread_pool()->Task<Pool> {
        Task::succeed(||Pool::new(num_cpus::get() as u32))
    }
}
impl Game for Main{
    type Input = ();
    type LoadingScreen = ();
    const TICKS_PER_SECOND: u16 = 60;
    fn load(_window: &Window) -> Task<Main>{
        (
            Task::stage("Initialising", Self::initialise_solver(), ),
            Task::stage("Loading Assets", Self::load_sprite()),
            Task::stage("Multithreading", Self::generate_thread_pool()),
        )
            .join()
            .map(|(solver, sprite,thread_pool)| Main {
                solver,
                batch: Batch::new(sprite),
                thread_pool
            })
    }
    fn update(&mut self, _window: &Window) {
        let now = SystemTime::now();
        let dt = 1.0/STEPS as f32;
        for s in 0..STEPS{
            unsafe {
                self.solver.update_objects(dt);
                if s%3 ==0{
                    self.solver.update_grid();
                }
                self.solver.solve_collisions_threaded(&mut self.thread_pool);
            }
        }
        println!("fps: {}",1000000./now.elapsed().unwrap().as_micros()as f32);
        if self.solver.object_count() < 97000 {
            let num = ((self.solver.object_count()+8000)/800).min(80);
            for i in 0..num{
                self.solver.new_object(15.+i as f32*0.1, 20. + i as f32*5.5,Vector::new(60.0, 0.0));
            }
        }

    }
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
        let scale = RADIUS/28.0;
        unsafe {
            let sprites = (*self.solver.objects().get()).par_iter().map(|object| {
                Sprite {
                    source: Rectangle {
                        x: 0,
                        y: 0,
                        width: 56,
                        height: 56,
                    },
                    position: Point::new(object.position().x - RADIUS, object.position().y - RADIUS),
                    scale: (scale, scale),
                }
            });
            self.batch.clear();
            self.batch.par_extend(sprites);
            self.batch.draw(&mut frame.as_target());
        }
    }
}