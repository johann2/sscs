#[macro_use]
extern crate simple_ecs;
use std::mem;

use simple_ecs::{World,Component,Entity,System};
static POSITION_MASK:u32=1<<1;
static SPEED_MASK:u32   =1<<2;
static TARGET_MASK:u32  =1<<3;

#[derive(Clone,Copy,PartialEq,PartialOrd)]
struct Vector2{x:f32,y:f32}

#[derive(Clone,PartialEq)]
pub struct Speed 
{
    val:Vector2
}

#[derive(Clone,PartialEq)]
pub struct Position
{
    val:Vector2
}

#[derive(Clone,PartialEq)]
pub struct Target
{
    val:Option<Entity>,
}

impl_entity_data!
{
	EntityData <()>
	{
		Speed:speeds:SPEED_MASK,
		Position:positions:POSITION_MASK,
		Target:targets:TARGET_MASK
	}
}


#[test]
fn test_id_recycle() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();
	let entity2=ecs.add_entity();

	assert!(ecs.entity_valid(&entity1));
	assert!(ecs.entity_valid(&entity2));

	ecs.delete_entity(&entity1);
	ecs.update(&systems);

	assert!(!ecs.entity_valid(&entity1));
	assert!(ecs.entity_valid(&entity2));

	let entity3=ecs.add_entity();

	assert!(!ecs.entity_valid(&entity1));
	assert!(ecs.entity_valid(&entity2));
	assert!(ecs.entity_valid(&entity3));
	//assert_eq!(entity3.version,1);

}

#[test]
fn test_invalidation() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();
	let entity2=ecs.add_entity();
	let entity1dup=entity1;

	assert!(ecs.entity_valid(&entity1dup));
	assert!(ecs.entity_valid(&entity1));
	assert!(ecs.entity_valid(&entity2));

	ecs.delete_entity(&entity1);
	ecs.update(&systems);

	assert!(!ecs.entity_valid(&entity1));
	assert!(!ecs.entity_valid(&entity1dup));
	assert!(ecs.entity_valid(&entity2));
}


#[test]
fn component_add() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();

	assert!(!Component::<Speed>::has(&ecs,&entity1));
	ecs.add(&entity1,&Speed{val:Vector2{x:0.0,y:1.0}});
	assert!(Component::<Speed>::has(&ecs,&entity1));
	ecs.update(&systems);

}


#[test]
fn component_remove() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();

	assert!(!Component::<Speed>::has(&ecs,&entity1));
	assert!(!Component::<Position>::has(&ecs,&entity1));
	
	ecs.add(&entity1,&Speed{val:Vector2{x:0.0,y:1.0}});
	
	assert!(Component::<Speed>::has(&ecs,&entity1));
	assert!(!Component::<Position>::has(&ecs,&entity1));
	
	ecs.add(&entity1,&Position{val:Vector2{x:0.0,y:1.0}});
	
	assert!(Component::<Speed>::has(&ecs,&entity1));
	assert!(Component::<Position>::has(&ecs,&entity1));

	Component::<Speed>::remove(&mut ecs,&entity1);
	assert!(!Component::<Speed>::has(&ecs,&entity1));
	assert!(Component::<Position>::has(&ecs,&entity1));
	ecs.update(&systems);
}

#[test]
fn component_remove_with_entity() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();
	ecs.add(&entity1,&Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity1,&Position{val:Vector2{x:0.0,y:1.0}});
	ecs.update(&systems);
	ecs.delete_entity(&entity1);


	let entity2=ecs.add_entity();
	assert!(!Component::<Speed>::has(&ecs,&entity2));
	assert!(!Component::<Position>::has(&ecs,&entity2));
	ecs.update(&systems);

}

#[derive(Clone,PartialEq)]
struct TestSystem;

impl System<EntityData,()> for TestSystem 
{
	fn process(&self,entities:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		let mut count=0;
		for e in entities.iter()
		{
			assert!(Component::<Speed>::has(world,&e));
			assert!(Component::<Position>::has(world,&e));
			count+=1;
		}
		assert_eq!(count,5);
	}

	fn get_interesting_entities(&self,world:&mut World<EntityData,()>)->Vec<Entity>
	{
		let mask=0|SPEED_MASK|POSITION_MASK;

		world.entities.iter().filter(|e| world.components[e.id]&mask==mask).map(|x|*x).collect::<Vec<Entity>>()
	}
}

#[derive(Clone,PartialEq)]
struct TestSystem2;

impl System<EntityData,()> for TestSystem2
{
	fn process(&self,interested:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		for e in interested.iter()
		{
			world.delete_entity(e);
			let entity=world.add_entity();
			world.add(&entity,&Target{val:None});
			break;
		}
	}

	fn get_interesting_entities(&self,world:&mut World<EntityData,()>)->Vec<Entity>
	{
		let mask=0|SPEED_MASK|POSITION_MASK;

		world.entities.iter().filter(|&e| world.components[e.id]&mask==mask).map(|x|*x).collect::<Vec<Entity>>()
	}
}

#[test]
fn system_filter() 
{
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let mut ecs:World<EntityData,()>=World::new();
	systems.push(Box::new(TestSystem));

	for _ in 0..4
	{
		let entity1=ecs.add_entity();
		ecs.add(&entity1,&Speed{val:Vector2{x:0.0,y:1.0}});
		ecs.add(&entity1,&Position{val:Vector2{x:0.0,y:1.0}});
		ecs.add_entity();
	}

	let entity2=ecs.add_entity();
	ecs.add(&entity2,&Speed{val:Vector2{x:0.0,y:1.0}});

	let entity3=ecs.add_entity();
	ecs.add(&entity3,&Position{val:Vector2{x:0.0,y:1.0}});

	let entity4=ecs.add_entity();
	ecs.add(&entity4,&Position{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity4,&Target{val:None});

	let entity5=ecs.add_entity();
	ecs.add(&entity5,&Target{val:None});

	let entity6=ecs.add_entity();
	ecs.add(&entity6,&Target{val:None});
	ecs.add(&entity6,&Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity6,&Position{val:Vector2{x:0.0,y:1.0}});

	ecs.update(&systems);
	ecs.update(&systems);
}

#[test]
fn system_add_remove_entities() 
{
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let mut ecs:World<EntityData,()>=World::new();
	systems.push(Box::new(TestSystem2));

	for _ in 0..4
	{
		let entity1=ecs.add_entity();
		ecs.add(&entity1,&Speed{val:Vector2{x:0.0,y:1.0}});
		ecs.add(&entity1,&Position{val:Vector2{x:0.0,y:1.0}});
		ecs.add_entity();
	}

	let entity2=ecs.add_entity();
	ecs.add(&entity2,&Speed{val:Vector2{x:0.0,y:1.0}});

	let entity3=ecs.add_entity();
	ecs.add(&entity3,&Position{val:Vector2{x:0.0,y:1.0}});

	let entity4=ecs.add_entity();
	ecs.add(&entity4,&Position{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity4,&Target{val:None});

	let entity5=ecs.add_entity();
	ecs.add(&entity5,&Target{val:None});

	let entity6=ecs.add_entity();
	ecs.add(&entity6,&Target{val:None});
	ecs.add(&entity6,&Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity6,&Position{val:Vector2{x:0.0,y:1.0}});

	for _ in 0..12
	{
		ecs.update(&systems);
	}

	for e in ecs.entities.iter()
	{
		if ecs.entity_valid(&e)
		{
			let success=!Component::<Position>::has(&ecs,&e) || !Component::<Speed>::has(&ecs,&e);
			assert!(success);
		}
	}
}
