#[macro_use]
extern crate simple_ecs;

use simple_ecs::{World,Entity,System,ComponentAccess};

#[derive(Clone,Copy,PartialEq,PartialOrd,Default)]
struct Vector2{x:f32,y:f32}

#[derive(Clone,PartialEq,Default)]
pub struct Speed 
{
    val:Vector2
}


#[derive(Clone,PartialEq,Default)]
pub struct Position
{
    val:Vector2
}


#[derive(Clone,PartialEq,Default)]
pub struct Target
{
    val:Option<Entity>,
}

#[derive(Clone,PartialEq,Default)]
pub struct Generic<T:Clone>
{
	val:T,
}

impl_entity_data!
{
	EntityData <()>
	{
		Speed:speeds:1<<1,
		Position:positions:1<<2,
		Target:targets:1<<3,
		Generic<usize>:generic_1:1<<4,
		Generic<u8>:generic_2:1<<5
	}
}

#[test]
fn test_id_recycle() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();
	let entity2=ecs.add_entity();

	assert!(ecs.entity_valid(&entity1));
	assert!(ecs.entity_valid(&entity2));

	ecs.delete_entity(&entity1);
	ecs.update(&mut systems);

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
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();
	let entity2=ecs.add_entity();
	let entity1dup=entity1;

	assert!(ecs.entity_valid(&entity1dup));
	assert!(ecs.entity_valid(&entity1));
	assert!(ecs.entity_valid(&entity2));

	ecs.delete_entity(&entity1);
	ecs.update(&mut systems);

	assert!(!ecs.entity_valid(&entity1));
	assert!(!ecs.entity_valid(&entity1dup));
	assert!(ecs.entity_valid(&entity2));
}


#[test]
fn component_add() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();

	assert!(!ecs.has::<Speed>(&entity1));
	ecs.add(&entity1,Speed{val:Vector2{x:0.0,y:1.0}});
	assert!(ecs.has::<Speed>(&entity1));
	ecs.update(&mut systems);

}


#[test]
fn component_remove() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();

	assert!(!ecs.has::<Speed>(&entity1));
	assert!(!ecs.has::<Position>(&entity1));
	
	ecs.add(&entity1,Speed{val:Vector2{x:0.0,y:1.0}});
	
	assert!(ecs.has::<Speed>(&entity1));
	assert!(!ecs.has::<Position>(&entity1));
	
	ecs.add(&entity1,Position{val:Vector2{x:0.0,y:1.0}});
	
	assert!(ecs.has::<Speed>(&entity1));
	assert!(ecs.has::<Position>(&entity1));

	ecs.remove::<Speed>(&entity1);
	assert!(!ecs.has::<Speed>(&entity1));
	assert!(ecs.has::<Position>(&entity1));
	ecs.update(&mut systems);
}

#[test]
fn component_remove_with_entity() 
{
	let mut ecs:World<EntityData,()>=World::new();
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let entity1=ecs.add_entity();
	ecs.add(&entity1,Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity1,Position{val:Vector2{x:0.0,y:1.0}});
	ecs.update(&mut systems);
	ecs.delete_entity(&entity1);


	let entity2=ecs.add_entity();
	assert!(!ecs.has::<Speed>(&entity2));
	assert!(!ecs.has::<Position>(&entity2));
	ecs.update(&mut systems);

}

#[derive(Clone,PartialEq)]
struct TestSystem;

impl System<EntityData,()> for TestSystem 
{
	fn process(&mut self,entities:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		let mut count=0;
		for e in entities.iter()
		{
			assert!(world.has::<Speed>(&e));
			assert!(world.has::<Position>(&e));
			count+=1;
		}
		assert_eq!(count,5);
	}

	fn get_entity_mask(&self)->u32
	{
		Speed::mask()|Position::mask()
	}
}

#[derive(Clone,PartialEq)]
struct TestSystem2;

impl System<EntityData,()> for TestSystem2
{
	fn process(&mut self,interested:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		for e in interested.iter()
		{
			world.delete_entity(e);
			let entity=world.add_entity();
			world.add(&entity,Target{val:None});
			break;
		}
	}

	fn get_entity_mask(&self)->u32
	{
		Speed::mask()|Position::mask()
	}
}

#[derive(Clone,PartialEq)]
struct MutableTestSystem {
	pub val:f32,
}

impl System<EntityData,()> for MutableTestSystem
{
	fn process(&mut self,interested:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		for e in interested.iter()
		{
			let mut position:&mut Position=world.get_mut(e).unwrap();
			position.val.x=self.val;
			position.val.y=self.val;
		}
		self.val+=1.0;
	}

	fn get_entity_mask(&self)->u32
	{
		Position::mask()
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
		ecs.add(&entity1,Speed{val:Vector2{x:0.0,y:1.0}});
		ecs.add(&entity1,Position{val:Vector2{x:0.0,y:1.0}});
		ecs.add_entity();
	}

	let entity2=ecs.add_entity();
	ecs.add(&entity2,Speed{val:Vector2{x:0.0,y:1.0}});

	let entity3=ecs.add_entity();
	ecs.add(&entity3,Position{val:Vector2{x:0.0,y:1.0}});

	let entity4=ecs.add_entity();
	ecs.add(&entity4,Position{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity4,Target{val:None});

	let entity5=ecs.add_entity();
	ecs.add(&entity5,Target{val:None});

	let entity6=ecs.add_entity();
	ecs.add(&entity6,Target{val:None});
	ecs.add(&entity6,Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity6,Position{val:Vector2{x:0.0,y:1.0}});

	ecs.update(&mut systems);
	ecs.update(&mut systems);
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
		ecs.add(&entity1,Speed{val:Vector2{x:0.0,y:1.0}});
		ecs.add(&entity1,Position{val:Vector2{x:0.0,y:1.0}});
		ecs.add_entity();
	}

	let entity2=ecs.add_entity();
	ecs.add(&entity2,Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity2,Generic::<usize>{val:10});

	let entity3=ecs.add_entity();
	ecs.add(&entity3,Position{val:Vector2{x:0.0,y:1.0}});

	let entity4=ecs.add_entity();
	ecs.add(&entity4,Position{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity4,Target{val:None});

	let entity5=ecs.add_entity();
	ecs.add(&entity5,Target{val:None});

	let entity6=ecs.add_entity();
	ecs.add(&entity6,Target{val:None});
	ecs.add(&entity6,Speed{val:Vector2{x:0.0,y:1.0}});
	ecs.add(&entity6,Position{val:Vector2{x:0.0,y:1.0}});

	for _ in 0..12
	{
		ecs.update(&mut systems);
	}

	for e in ecs.entities().iter()
	{
		if ecs.entity_valid(&e)
		{
			let success=!ecs.has::<Position>(&e) || !ecs.has::<Speed>(&e);
			assert!(success);
		}
	}
}

#[test]
fn mutable_system() 
{
	let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
	let mut ecs:World<EntityData,()>=World::new();
	systems.push(Box::new(MutableTestSystem{val:0.0}));

	for _ in 0..10
	{
		let entity=ecs.add_entity();
		ecs.add(&entity,Position{val:Vector2{x:0.0,y:1.0}});
	}

	for _ in 0..12
	{
		ecs.update(&mut systems);
	}

	for e in ecs.entities().iter()
	{
		if ecs.entity_valid(&e)
		{
			let success=if let Some(pos)=ecs.get::<Position>(&e) {
				println!("{:?},{:?},",pos.val.x,pos.val.y);
				pos.val.x==11.0 && pos.val.y==11.0
			} else {false};
			assert!(success);
		}
	}
}
