# sscs
Simple entity-component library inspired by [ecs-rs](https://github.com/HeroesGrave/ecs-rs) and [entityx](https://github.com/alecthomas/entityx) for Rust. I've only tested it on nightly builds, but it should work on stable, too.

###Defining features:
* Simple
* Entity is an index to component table
* Everything is kept in ```Vec```s
* Uses bitmasks to keep track which components an entity has. This limits the number of component types to 31, but should be really fast (haven't checked yet, though)

##Tutorial
There's a working example in the examples folder, if something is too confusing here.

###Importing
Add to cargo.toml:
```
[dependencies.simple_ecs]
git = "https://github.com/johann2/sscs.git"
```

Add to your crate root:
`extern crate simple_ecs`

###Defining your components
Components must implement `Default` and every component type must have an unique name
So, for example:
```rust
pub struct Speed {
    val:(f32,f32)
}

pub struct Position {
    val:(f32,f32)
}
```

###Registering Components
A macro implements most of the code needed for accessing components and entities.
The last column, bitmask has to be a power of two and unique for every component.
For components defined above, it would look like this:
```rust
impl_entity_data! {
	EntityData <()>	{
		//Class name	:database field name	:bitmask
		Speed		:speeds			:1<<1,
		Position	:positions		:1<<2,
	}
}
```
The type argument, in this example `()` is for a struct that holds data not directly related to any entity, like level geometry.

### Manipulating entities


```rust
let mut ecs:World<EntityData,()>=World::new(); //Create a new world

//manipulating entities:
let entity=ecs.add_entity();                  //Create a new entity
assert!(ecs.entity_valid(&entity);            //Check if an entity is valid
ecs.remove_entity(&entity);                   //Remove an entity.

//Checking if an entity is valid is important, because the following functions will panic if it isn't:
//(let's assume `entity` didn't get deleted yet)

ecs.add(&entity,Position{val:(10.0,0.0)});   //Add a new Position component
ecs.add(&entity,Speed{val:(0.0,0.0)});       //          Speed

//If a component exists, `add` overwrites it. An entity can only have one of every type of component.

//get an optional reference to a component:
let position=ecs.get::<Position>(&entity); 

//check if an entity has a component
assert!(ecs.has::<Speed>(&entity)); 

//Remove a component
ecs.remove::<Speed>(&entity);

```

###Systems
Systems need to implement the `System` trait. Systems aren't serializable, so they should only contain data for caching and stuff like that.

```rust
struct MovementSystem;

impl System<EntityData,()> for MovementSystem
{
	//processing function, recieves a list of entities that match the bitmask returned by get_entity_mask
	fn process(&self,entities:Vec<Entity>,world:&mut World<EntityData,()>)	{
		for e in entities.iter() {
			//world.componentdata.{field name}[entity.id] is a faster way to access components, 
			//but really unsafe.	
			let position=world.componentdata.positions[e.id].val;
			let speed=world.componentdata.speeds[e.id].val;
			world.componentdata.positions[e.id].val=(position.0+speed.0,position.1+speed.1);
			//you can also delete and create components and entities here without any problems
		}
	}

	fn get_entity_mask(&self,world:&mut World<EntityData,()>)->Vec<Entity>	{
		Speed::mask()|Position::mask()
	}
}
```

###Main loop and setup

```rust

let mut ecs:World<EntityData,()>=World::new();
//Systems are stored as boxed trait objects in a vector.
let mut systems:Vec<Box<System<EntityData,()>>>=Vec::new();
//Adding a system:
systems.push(Box::new(MovementSystem));

//Add/setup entities (see above)
...

//Main loop

loop {
  world.update(&systems);
}
```
