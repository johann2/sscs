# simple-ecs
Simple entity-component library inspired by [ecs-rs](https://github.com/HeroesGrave/ecs-rs) and [entityx](https://github.com/alecthomas/entityx) for Rust

Currently runs only on nightly builds.

###Main Points:
* Simple
* Entity is an index to component table
* Everything is kept in ```Vec```s
* Uses bitmasks to keep track which components an entity has. This limits the number of component types to 31, but should be really fast (haven't checked yet, though)
* Systems have no state

##Tutorial
There's also a working example in examples folder, if something is too confusing here.

###Importing
Add to cargo.toml:
```
[dependencies.simple_ecs]
git = "https://github.com/johann2/simple-ecs.git"
```

Add to your crate root:
`extern crate simple_ecs`

###Defining your components
Components must implement `Clone` and `Default` and every component type must have an unique name
So, for example:
```rust
#[derive(Clone)]
pub struct Speed 
{
    val:(f32,f32)
}

#[derive(Clone)]
pub struct Position
{
    val:(f32,f32)
}
```

###Defining component bitmasks
Unfortunately, it's impossible to write a macro that defines bitmasks, so you have to define them manually.
They are used mostly internally to filter out entities with specific components.

A good name is your component name in uppercase+_MASK.
```
static POSITION_MASK:u32=1<<1;
static SPEED_MASK:u32   =1<<2;
```

###Registering Components
A macro implements most of the code needed for accessing components and entities.
For components defined above, it would look like this:

```rust
impl_entity_data!
{
	EntityData <()>
	{
    //Class name:database field name:bitmask name
		Speed:speeds:SPEED_MASK,
		Position:positions:POSITION_MASK,
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

ecs.add(&entity,&Position{val:(10.0,0.0)});   //Add a new Position component
ecs.add(&entity,&Speed{val:(0.0,0.0)});       //          Speed
//if a component exists, `add` overwrites it.

//get a copy of a component, wrapped in an Option
let position=Component::<Position>::get(&ecs,&entity); 

//check if an entity has a component
//these function calls look a bit off, but I'm not sure how to improve them
assert!(!Component::<Speed>::has(&ecs,&entity)); 

//Remove a component
Component::<Speed>::remove(&ecs,&entity);

```

###Systems
Systems need to implement the `System` trait
For now, they are immutable and shouldn't really contain any data.

```rust
struct MovementSystem;

impl System<EntityData,()> for MovementSystem
{
  //processing function, recieves a list of entities to process and hopefully does it.
	fn process(&self,entities:Vec<Entity>,world:&mut World<EntityData,()>)
	{
		for e in entities.iter()
		{
      //world.componentdata.{field name}[entity.id] is a faster way to access components, but really unsafe.
			let position=world.componentdata.positions[e.id].val;
			let speed=world.componentdata.speeds[e.id].val;
			world.componentdata.positions[e.id].val=(position.0+speed.0,position.1+speed.1);
			//you can also delete and create components and entities here without any problems
		}
	}

  //this function filters all the entities for processing.
  //in this case, it selects all the entities that have the `Speed` and `Position` component.
	fn get_interesting_entities(&self,world:&mut World<EntityData,()>)->Vec<Entity>
	{
		let mask=0|SPEED_MASK|POSITION_MASK;
		world.entities.iter().filter(|&e| world.components[e.id]&mask==mask).map(|x|*x).collect::<Vec<Entity>>()
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

```rust
loop 
{
  world.update(&systems);
}
```
