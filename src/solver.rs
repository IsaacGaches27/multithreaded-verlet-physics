/*use coffee::graphics::{Point, Vector};
use std::cell::UnsafeCell;
use std::ops::Range;
use std::sync::Arc;
use scoped_threadpool::Pool;
use std::sync::atomic::{AtomicPtr, Ordering};
use rayon::prelude::*;
use std::time::SystemTime;
use super::{WIDTH,HEIGHT,RADIUS,GRAVITY};
pub struct Object{
    pub position: Point,
    pre_position: Point,
    acceleration: Vector,
    id: usize,
}
pub struct Solver {
    pub objects: UnsafeCell<Vec<Object>>,
    cells: UnsafeCell<Vec<Vec<AtomicPtr<Vec<usize>>>>>,
    pub object_count:usize,
}
unsafe impl Send for Solver{}
unsafe impl Sync for Solver{}
impl Solver{
    pub fn new()->Self{
        let mut cells= Vec::new();//UnsafeCell::new(vec![vec![Vec::new();(HEIGHT as f32/RADIUS) as usize];(WIDTH as f32/RADIUS) as usize]);
        for x in 0..(WIDTH as f32/RADIUS) as usize{
            cells.push(Vec::new());
            for _y in 0..(HEIGHT as f32/RADIUS) as usize{
                cells[x].push(AtomicPtr::new(&mut Vec::new()));
            }
        }
        let cells = UnsafeCell::new(cells);
        Self{
            objects: UnsafeCell::new(Vec::new()),
            cells,
            object_count:0,
        }
    }
    pub unsafe fn solve_collisions_threaded(&mut self, pool:&mut Pool){
        let arc_self = Arc::new(self);
        pool.scoped(|s|{
            for i in 0..num_cpus::get(){
                let arc_self = Arc::clone(&arc_self);
                s.execute(move||{
                    Self::solve_collisions(
                        &arc_self,
                        i*2*((WIDTH as f32 / RADIUS) as usize/(num_cpus::get()*2))..((i*2)+1)*((WIDTH as f32 / RADIUS) as usize/(num_cpus::get()*2))
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
                        ((i*2)+1)*((WIDTH as f32/RADIUS)as usize/(num_cpus::get()*2))..((i*2)+2)*((WIDTH as f32/RADIUS)as usize/(num_cpus::get()*2))
                    );
                });
            }
        });
    }
    unsafe fn solve_collisions(arc_self: &Arc<&mut Solver>,range:Range<usize>){
        for x in range{
            for y in 0..(HEIGHT as f32 / RADIUS) as usize {
                for k in 0..(**(*arc_self.cells.get())[x][y].get_mut()).len(){
                    let object_id = (**(*arc_self.cells.get())[x][y].get_mut())[k];
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
        for j in 0..(**(*cell_ptr)[cell_x][cell_y].get_mut()).len(){
            if object_id == (**(*cell_ptr)[cell_x][cell_y].get_mut())[j] { continue; }
            let collision_axis = (*obj_ptr)[object_id].position - (*obj_ptr)[(**(*cell_ptr)[cell_x][cell_y].get_mut())[j]].position;
            let distance = (collision_axis.x * collision_axis.x + collision_axis.y * collision_axis.y).sqrt();
            let minimum_distance = RADIUS * 2.0;
            if distance < minimum_distance {
                let n = collision_axis / distance;
                let delta = minimum_distance - distance;
                (*obj_ptr)[object_id].position += n * delta * 0.5;
                (*obj_ptr)[(**(*cell_ptr)[cell_x][cell_y].get_mut())[j]].position -= n * delta * 0.5;
            }
        }
    }
    pub unsafe fn update_positions(&mut self,dt:f32){
        let (max_width,max_height) = (WIDTH as f32-RADIUS,HEIGHT as f32 -RADIUS);
        (*self.objects.get()).par_iter_mut().for_each(move|object|{
            object.acceleration.y += GRAVITY;
            let velocity = object.position - object.pre_position;
            object.pre_position = object.position;
            object.position += velocity + object.acceleration * dt * dt;
            object.acceleration.fill(0.);
            object.position.y = object.position.y.clamp(RADIUS,max_height);
            object.position.x = object.position.x.clamp(RADIUS,max_width);

        });
    }
    pub unsafe fn set_cells(&self){
        (*self.cells.get()).par_iter_mut().for_each(|cellsx|cellsx.par_iter_mut().for_each(|cellsy|(**cellsy.get_mut()).clear()));
        let div = 1./(RADIUS * 2.0);
        (*self.objects.get()).iter().for_each(move|object|{
            let cell_pos = object.position * div;
            (**(*self.cells.get())[cell_pos.x as usize][cell_pos.y as usize].get_mut()).push(object.id);
        });
    }
    pub fn new_object(&mut self,x:f32,y:f32){
        unsafe{
            (*self.objects.get()).push(Object{
            position: Point::new(x,y),
            pre_position: Point::new(x-1.4,y),
            acceleration: Vector::new(0.0, 0.0),
            id: self.object_count,})
        }
        self.object_count+=1;
    }
}*/
use coffee::graphics::{Point, Vector};
use std::cell::UnsafeCell;
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use scoped_threadpool::Pool;
use rayon::prelude::*;
use std::time::{Duration, SystemTime};
use super::{WIDTH,HEIGHT,RADIUS,GRAVITY};
pub struct Object{
    pub position: Point,
    pre_position: Point,
    acceleration: Vector,
    id: usize,
}
pub struct Solver {
    pub objects: UnsafeCell<Vec<Object>>,
    cells: Vec<Vec<Lock<Vec<usize>>>>,
    pub object_count:usize,
}
struct Lock<T>{
    locked: AtomicBool,
    data: UnsafeCell<T>
}
impl<T> Lock<T> {
    fn lock(&self)->*mut T{
        while self.locked.load(Ordering::SeqCst) {

        }
        self.locked.store(true,Ordering::SeqCst);
        self.data.get()
    }
    fn unlock(&self){
        self.locked.store(false,Ordering::SeqCst);
    }
    fn get(&self)->*mut T{
        self.data.get()
    }
    fn new(data: T)->Self{
        Self{
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}
unsafe impl<T> Send for Lock<T>{}
unsafe impl<T> Sync for Lock<T>{}
unsafe impl Send for Solver{}
unsafe impl Sync for Solver{}
impl Solver{
    pub fn new()->Self{
        let mut cells= Vec::new();
        for x in 0..(WIDTH as f32/RADIUS) as usize{
            Lock::new(cells.push(Vec::new()));
            for _y in 0..(HEIGHT as f32/RADIUS) as usize{
                cells[x].push(Lock::new(Vec::new()));
            }
        }
        Self{
            objects: UnsafeCell::new(Vec::new()),
            cells,
            object_count:0,
        }
    }
    pub unsafe fn solve_collisions_threaded(&mut self, pool:&mut Pool){
        let arc_self = Arc::new(self);
        pool.scoped(|s|{
            for i in 0..num_cpus::get(){
                let arc_self = Arc::clone(&arc_self);
                s.execute(move||{
                    Self::solve_collisions(
                        &arc_self,
                        i*2*((WIDTH as f32 / RADIUS) as usize/(num_cpus::get()*2))..((i*2)+1)*((WIDTH as f32 / RADIUS) as usize/(num_cpus::get()*2))
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
                        ((i*2)+1)*((WIDTH as f32/RADIUS)as usize/(num_cpus::get()*2))..((i*2)+2)*((WIDTH as f32/RADIUS)as usize/(num_cpus::get()*2))
                    );
                });
            }
        });
    }
    unsafe fn solve_collisions(arc_self: &Arc<&mut Solver>,range:Range<usize>){
        for x in range{
            for y in 0..(HEIGHT as f32 / RADIUS) as usize {
                for k in 0..unsafe{(*(*arc_self.cells)[x][y].get()).len()}{
                    let object_id = unsafe{(*(*arc_self.cells)[x][y].get())[k]};
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
        let cell_ptr = &self.cells;
        let obj_ptr = self.objects.get();
        for j in 0..(*(*cell_ptr)[cell_x][cell_y].get()).len(){
            if object_id == (*(*cell_ptr)[cell_x][cell_y].get())[j] { continue; }
            let collision_axis = (*obj_ptr)[object_id].position - (*obj_ptr)[(*(*cell_ptr)[cell_x][cell_y].get())[j]].position;
            let distance = (collision_axis.x * collision_axis.x + collision_axis.y * collision_axis.y).sqrt();
            let minimum_distance = RADIUS * 2.0;
            if distance < minimum_distance {
                let n = collision_axis / distance;
                let delta = minimum_distance - distance;
                (*obj_ptr)[object_id].position += n * delta * 0.5;
                (*obj_ptr)[(*(*cell_ptr)[cell_x][cell_y].get())[j]].position -= n * delta * 0.5;
            }
        }
    }
    pub unsafe fn update_positions(&mut self,dt:f32){
        let (max_width,max_height) = (WIDTH as f32-RADIUS,HEIGHT as f32 -RADIUS);
        (*self.objects.get()).par_iter_mut().for_each(move|object|{
            object.acceleration.y += GRAVITY;
            let velocity = object.position - object.pre_position;
            object.pre_position = object.position;
            object.position += velocity + object.acceleration * dt * dt;
            object.acceleration.fill(0.);
            object.position.y = object.position.y.clamp(RADIUS,max_height);
            object.position.x = object.position.x.clamp(RADIUS,max_width);
        });
    }
    /*pub unsafe fn set_cells(&self){
        (*self.cells.get()).par_iter_mut().for_each(|cells|cells.par_iter_mut().for_each(|cells|cells.clear()));
        let div = 1./(RADIUS * 2.0);
        (*self.objects.get()).iter().for_each(move|object|{
            let cell_pos = object.position * div;
            (*self.cells.get())[cell_pos.x as usize][cell_pos.y as usize].push(object.id);
        });
    }*/
    pub unsafe fn set_cells(&mut self){
        (*self.cells).par_iter_mut().for_each(|cellsx|cellsx.par_iter_mut().for_each(|cellsy|(*cellsy.get()).clear()));
        let div = 1./(RADIUS * 2.0);
        (*self.objects.get()).iter().for_each(move|object|{
            let cell_pos = object.position * div;
            let mut ptr = self.cells[cell_pos.x as usize][cell_pos.y as usize].lock();
            (*ptr).push(object.id);
            //add to start or end to make deterministic;
            self.cells[cell_pos.x as usize][cell_pos.y as usize].unlock();
        });
    }
    pub fn new_object(&mut self,x:f32,y:f32){
        unsafe{
            (*self.objects.get()).push(Object{
            position: Point::new(x,y),
            pre_position: Point::new(x-1.4,y),
            acceleration: Vector::new(0.0, 0.0),
            id: self.object_count,})
        }
        self.object_count+=1;
    }
}