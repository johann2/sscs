#[macro_use]
extern crate sscs;
extern crate rand;
extern crate ncurses;

use sscs::{System,World,Entity,ComponentAccess};
use std::iter;
use std::iter::FromIterator;


//Helper functions
pub fn init_2d_vec<T:Clone>(width:usize,height:usize,default_value:T) -> Vec<Vec<T>> {
    let col=Vec::from_iter(iter::repeat(default_value).take(height));
    Vec::from_iter(iter::repeat(col).take(width))
}

fn random_vector(mult:f32)->(f32,f32) {
	(rand::random::<f32>()*mult-mult/2.0,rand::random::<f32>()*mult-mult/2.0)
}

//Component structs

#[derive(Clone,PartialEq,Default)]
pub struct Speed {
    val:(f32,f32)
}

#[derive(Clone,PartialEq,Default)]
pub struct Position {
    val:(f32,f32)
}

#[derive(Clone,PartialEq,Default)]
pub struct Character {
    val:char
}


//This generates all the methods for adding,removing and accessing individual components
//between {} is a list of components.
impl_entity_data! {
	EntityData {
 //		v component type
 			  //v component array name;
 			  		//v component mask
		Speed:speeds:1<<1,
		Position:positions:1<<2,
		Character:characters:1<<3
	}
}

type ExampleWorld = World<EntityData,()>;
 
//Implement movementsystem
struct MovementSystem;

impl System<ExampleWorld> for MovementSystem {
	fn process(&mut self,entities:Vec<Entity>,world:&mut ExampleWorld) {
		for e in entities.iter() {
			let position=world.component_data.positions[e.id()].val;
			let speed=world.component_data.speeds[e.id()].val;
			world.component_data.positions[e.id()].val=(position.0+speed.0,position.1+speed.1);
			if position.0 < -30.0 || position.0>30.0 || position.1 < -30.0 || position.1>30.0 {
				world.delete_entity(e);

				let entity=world.add_entity();
				world.add(&entity,Speed{val:random_vector(0.02)});
				world.add(&entity,Position{val:random_vector(10.0)});
				world.add(&entity,Character{val:'o'});
			}
		}
	}

	fn get_entity_mask(&self)->u32 {
		Speed::mask()|Position::mask()
	}
}

struct RenderSystem;

impl System<ExampleWorld> for RenderSystem {
	fn process(&mut self,entities:Vec<Entity>,world:&mut ExampleWorld) {
//		world.globaldata.display=init_2d_vec(30,30,' ');

		for e in entities.iter() {
			let position=world.component_data.positions[e.id()].val;
			let character=world.component_data.characters[e.id()].val;
			let x=position.0 as i32+30;
			let y=position.1 as i32+30;

			ncurses::mvaddch(y,x,character as u64);
		}
	}

	fn get_entity_mask(&self)->u32 {
		Character::mask()|Position::mask()
	}
}


fn main() {
	ncurses::initscr();

	let mut movement_system=MovementSystem;
	let mut render_system=RenderSystem;

	let mut world=ExampleWorld::new();
	let mut systems=Vec::new();
	systems.push(&mut movement_system as &mut System<ExampleWorld>);
	systems.push(&mut render_system as &mut System<ExampleWorld>);


	//Add moving objects
	for _ in 0..100 {
		let entity=world.add_entity();
	
		world.add(&entity,Speed{val:random_vector(0.01)});
		world.add(&entity,Position{val:random_vector(10.0)});
		world.add(&entity,Character{val:'o'});
	}

	//Add static objects
	for _ in 0..10 {
		let entity=world.add_entity();
		world.add(&entity,Position{val:(random_vector(15.0))});
		world.add(&entity,Character{val:'#'});
	}


	loop {
		ncurses::erase();
		world.update(&mut systems);
		ncurses::refresh();
	}
}
