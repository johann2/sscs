# sscs (super-simple component system)
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
[dependencies.sscs]
git = "https://github.com/johann2/sscs.git"
```

Add to your crate root:
`extern crate sscs`

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
	EntityData {
		//Class name	:database field name	:bitmask
		Speed		:speeds			:1<<1,
		Position	:positions		:1<<2,
	}
}
```

### Manipulating entities


```rust

//Let's define a type alias to keep examples clean.
type ExampleWorld=World<EntityData,()>;

let mut world=ExampleWorld::new(); 				//Create a new world

//manipulating entities:
let entity=world.add_entity();                  //Create a new entity
assert!(world.entity_valid(&entity);            //Check if an entity is valid
world.remove_entity(&entity);                   //Remove an entity.

//Checking if an entity is valid is important, because the following functions will panic if it isn't:
//(let's assume `entity` didn't get deleted yet)

world.add(&entity,Position{val:(10.0,0.0)});   //Add a new Position component
world.add(&entity,Speed{val:(0.0,0.0)});       //          Speed

//If a component exists, `add` overwrites it. An entity can have only one of every type of component.

//get an optional reference to a component:
let position=world.get::<Position>(&entity);

//check if an entity has a component
assert!(world.has::<Speed>(&entity));

//Remove a component
world.remove::<Speed>(&entity);

```

###Systems
Systems need to implement the `System` trait. Systems aren't serializable, so they should only contain data for caching and stuff like that.

```rust
struct MovementSystem;

impl System<ExampleWorld> for MovementSystem
{
	fn process(&self,entities:Vec<Entity>,world:&mut ExampleWorld)	{
		for e in entities.iter() {
			//world.component_data.{field name}[entity.id] is a faster way to access components,
			//but really unsafe.
			let position=world.component_data.positions[e.id].val;
			let speed=world.component_data.speeds[e.id].val;
			world.component_data.positions[e.id].val=(position.0+speed.0,position.1+speed.1);
			//you can also delete and create components and entities here without any problems
		}
	}

	fn get_entity_mask()->u32	{
		Speed::mask()|Position::mask()
	}

	//Optionally, you can also implement functions that deal with added and removed entities

	fn process_added(&mut self,entities:Vec<Entity>,world:&mut ExampleWorld) {
		for e in entities.iter() {
			println("Added:{:?}",e);
		}
	}

	fn process_removed(&mut self,entities:Vec<Entity>,world:&mut ExampleWorld) {
		for e in entities.iter() {
			println("Removed:{:?}",e);
		}
	}

}
```

###Main loop and setup

```rust

let mut sscs=ExampleWorld::new();
//Systems are stored as mutable trait objects in a vector.
//You are responsible for storing systems.
let mut movement_system=MovementSystem;

let mut systems:Vec<&mut System<ExampleWorld>>=Vec::new();
systems.push(&mut movement_system as &mut System<ExampleWorld>);

//Add/setup entities (see above)
...

//Main loop

loop {
  world.update(&systems);
}
```
