use coffee::graphics::{Point, Vector};
use std::cell::UnsafeCell;
use std::ops::Range;
use std::sync::Arc;
use scoped_threadpool::Pool;
use rayon::prelude::*;
use super::{WIDTH, HEIGHT, RADIUS, GRAVITY, OBJECTS_PER_CELL};

const CELLS_WIDTH: usize = (WIDTH as f32/RADIUS) as usize;
const CELLS_HEIGHT: usize = (HEIGHT as f32/RADIUS) as usize;

pub struct Object{
    position: Point,
    pre_position: Point,
    acceleration: Vector,
    id: usize,
}
impl Object{
    fn new(x: f32 ,y: f32, id: usize, acceleration: Vector) -> Self{
        Self {
            position: Point::new(x,y),
            pre_position: Point::new(x,y),
            acceleration,
            id,
        }
    }
    fn update(&mut self,dt:f32){
        self.acceleration.y += GRAVITY;
        let velocity = self.position - self.pre_position;
        self.pre_position = self.position;
        self.position += velocity + self.acceleration * dt * dt;
        self.acceleration.fill(0.);
        self.position.y = self.position.y.clamp(RADIUS,HEIGHT as f32 - RADIUS);
        self.position.x = self.position.x.clamp(RADIUS,WIDTH as f32 - RADIUS);
    }
    pub fn position(&self) -> &Point{
        &self.position
    }
}
#[derive(Copy, Clone)]
struct Cell {
    objects: [usize;OBJECTS_PER_CELL],
    count: usize,
}
impl Cell{
    fn new() -> Self{
        Self{
            objects: [0;OBJECTS_PER_CELL],
            count: 0,
        }
    }
    fn clear(&mut self){
        self.count = 0;
    }
    fn add_object(&mut self, object_id: usize){
        assert!(self.count < OBJECTS_PER_CELL);
        self.objects[self.count] = object_id;
        self.count+=1;
    }
}
pub struct Grid {
    objects: UnsafeCell<Vec<Object>>,
    cells: UnsafeCell<Vec<Cell>>,
    object_count:usize,
}
unsafe impl Send for Grid{}
unsafe impl Sync for Grid{}
impl Grid{
    pub fn new()->Self{
        let cells = (0..CELLS_WIDTH * CELLS_HEIGHT).map(|_|{
            Cell::new()
        }).collect::<Vec<_>>();

        Self{
            objects: UnsafeCell::new(Vec::with_capacity(100000)),
            cells: UnsafeCell::new(cells),
            object_count: 0,
        }
    }
    pub fn object_count(&self)->usize{
        self.object_count
    }
    pub fn objects(&self) -> &UnsafeCell<Vec<Object>>{
        &self.objects
    }
    pub unsafe fn update_grid(&mut self){
        (*self.cells.get()).par_iter_mut().for_each(|cell| cell.clear());

        let div = 1./(RADIUS* 2.0);

        (*self.objects.get()).iter().for_each(|object|{
            let cell = object.position*div;
            let i = cell.x as usize * CELLS_HEIGHT + cell.y as usize;
            (*self.cells.get())[i].add_object(object.id);
        });
    }
    pub unsafe fn update_objects(&mut self, dt: f32){
        (*self.objects.get()).par_iter_mut().for_each(|object| object.update(dt));
    }
    pub fn new_object(&mut self,x:f32,y:f32,acceleration: Vector){
        unsafe {
            (*self.objects.get()).push(Object::new(x, y, self.object_count,acceleration));
        }
        self.object_count+=1;
    }
    pub unsafe fn solve_collisions_threaded(&mut self, pool:&mut Pool){
        let arc_self = Arc::new(self);
        pool.scoped(|s|{
            for i in 0..num_cpus::get(){
                let arc_self = Arc::clone(&arc_self);
                s.execute(move||{
                    Self::solve_collisions(
                        &arc_self,
                        i*2*(CELLS_WIDTH/(num_cpus::get()*2))..((i*2)+1)*(CELLS_WIDTH/(num_cpus::get()*2))
                    );
                });
            }

        });
        pool.scoped(|s| {
            for i in 0..num_cpus::get() {
                let arc_self = Arc::clone(&arc_self);
                s.execute(move || {
                    Self::solve_collisions(
                        &arc_self,
                        ((i*2)+1)*(CELLS_WIDTH/(num_cpus::get()*2))..((i*2)+2)*(CELLS_WIDTH/(num_cpus::get()*2))
                    );
                });
            }
        });
    }
    unsafe fn solve_collisions(arc_self: &Arc<&mut Self>,range:Range<usize>){
        for x in range{
            for y in 0..CELLS_HEIGHT {
                for k in 0..(*arc_self.cells.get())[x * CELLS_HEIGHT + y].count{
                    let object_id = (*arc_self.cells.get())[x * CELLS_HEIGHT + y].objects[k];
                    Self::check_cell(&arc_self, y+1,x+1, object_id);
                    Self::check_cell(&arc_self, y+1,x, object_id);
                    Self::check_cell(&arc_self, y+1,x.saturating_sub(1), object_id);
                    Self::check_cell(&arc_self, y,x+1, object_id);
                    Self::check_cell(&arc_self, y,x, object_id);
                    Self::check_cell(&arc_self, y,x.saturating_sub(1), object_id);
                    Self::check_cell(&arc_self, y.saturating_sub(1),x+1, object_id);
                    Self::check_cell(&arc_self, y.saturating_sub(1),x, object_id);
                    Self::check_cell(&arc_self, y.saturating_sub(1),x.saturating_sub(1), object_id);
                }
            }
        }
    }
    unsafe fn check_cell(&self,cell_y:usize,cell_x: usize,object_id: usize){
        let cell_ptr = self.cells.get();
        let obj_ptr = self.objects.get();
        for j in 0..(*cell_ptr)[cell_x * CELLS_HEIGHT + cell_y].count{
            if object_id == (*cell_ptr)[cell_x * CELLS_HEIGHT + cell_y].objects[j] { continue; }
            let collision_axis = (*obj_ptr)[object_id].position - (*obj_ptr)[(*cell_ptr)[cell_x * CELLS_HEIGHT + cell_y].objects[j]].position;
            let distance = (collision_axis.x * collision_axis.x + collision_axis.y * collision_axis.y).sqrt();
            let minimum_distance = RADIUS * 2.0;
            if distance < minimum_distance {
                let n = collision_axis / distance;
                let delta = minimum_distance - distance;
                (*obj_ptr)[object_id].position += n * delta * 0.5;
                (*obj_ptr)[(*cell_ptr)[cell_x * CELLS_HEIGHT + cell_y].objects[j]].position -= n * delta * 0.5;
            }
        }
    }
}
