#[macro_use]
extern crate simple_ecs;
extern crate rand;
extern crate ncurses;

use simple_ecs::{System,World,Entity,Component};
use std::iter;
use std::iter::FromIterator;


//Helper functions
pub fn init_2d_vec<T:Clone>(width:usize,height:usize,default_value:T) -> Vec<Vec<T>>
{
    let col=Vec::from_iter(iter::repeat(default_value).take(height));
    Vec::from_iter(iter::repeat(col).take(width))
}

fn random_vector(mult:f32)->(f32,f32)
{
	(rand::random::<f32>()*mult-mult/2.0,rand::random::<f32>()*mult-mult/2.0)
}

//Component structs

#[derive(Clone,PartialEq,Default)]
pub struct Speed 
{
    val:(f32,f32)
}

#[derive(Clone,PartialEq,Default)]
pub struct Position
{
    val:(f32,f32)
}

#[derive(Clone,PartialEq,Default)]
pub struct Character
{
    val:char
}


//This generates all the methods for adding,removing and accessing individual components
//Type argument is for a struct that holds data that's not related to any entity, for example collision information or tiles
//In this case, it's empty
//between {} is a list of components.
impl_entity_data!
{
	EntityData <()>
	{
 //		v component type
 			  //v component array name;
		Speed:speeds:SPEED_MASK,
		Position:positions:POSITION_MASK,
		Character:characters:CHARACTER_MASK
	}
}

//Masks need to be defined manually, for now.
static POSITION_MASK:u32    =1<<1;
static SPEED_MASK:u32       =1<<2;
static CHARACTER_MASK:u32   =1<<3;


//Implement movementsystem
struct MovementSystem;

impl System<EntityData,()> for MovementSystem
{
	fn process(&self,entities:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		for e in entities.iter()
		{
			let position=world.componentdata.positions[e.id].val;
			let speed=world.componentdata.speeds[e.id].val;
			world.componentdata.positions[e.id].val=(position.0+speed.0,position.1+speed.1);
			if position.0 < -30.0 || position.0>30.0 || position.1 < -30.0 || position.1>30.0 
			{
				world.delete_entity(e);

				let entity=world.add_entity();
				world.add(&entity,&Speed{val:random_vector(0.01)});
				world.add(&entity,&Position{val:random_vector(10.0)});
				world.add(&entity,&Character{val:'o'});
			}
		}
	}

	fn get_interesting_entities(&self,world:&mut World<EntityData,()>)->Vec<Entity>
	{
		let mask=0|SPEED_MASK|POSITION_MASK;
		world.entities.iter().filter(|&e| world.components[e.id]&mask==mask).map(|x|*x).collect::<Vec<Entity>>()
	}
}

struct RenderSystem;

impl System<EntityData,()> for RenderSystem
{
	fn process(&self,entities:Vec<Entity>,world:&mut World<EntityData,()>)
	{
//		world.globaldata.display=init_2d_vec(30,30,' ');

		for e in entities.iter()
		{
			let position=world.componentdata.positions[e.id].val;
			let character=world.componentdata.characters[e.id].val;
			let x=position.0 as i32+30;
			let y=position.1 as i32+30;

			ncurses::mvaddch(y,x,character as u64);
		}
	}

	fn get_interesting_entities(&self,world:&mut World<EntityData,()>)->Vec<Entity>
	{
		let mask=0|CHARACTER_MASK|POSITION_MASK;
		world.entities.iter().filter(|&e| world.components[e.id]&mask==mask).map(|x|*x).collect::<Vec<Entity>>()
	}
}


fn main() 
{
	ncurses::initscr();

	let mut world:World<EntityData,()>=World::new();
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	systems.push(Box::new(MovementSystem));
	systems.push(Box::new(RenderSystem));


	//Add moving objects
	for _ in 0..100
	{
		let entity=world.add_entity();
	
		world.add(&entity,&Speed{val:random_vector(0.01)});
		world.add(&entity,&Position{val:random_vector(10.0)});
		world.add(&entity,&Character{val:'o'});
	}

	//Add static objects
	for _ in 0..10
	{
		let entity=world.add_entity();
		world.add(&entity,&Position{val:(random_vector(15.0))});
		world.add(&entity,&Character{val:'#'});
	}


	loop 
	{
		ncurses::erase();
		world.update(&systems);
		ncurses::refresh();
	}
}
