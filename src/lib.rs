#[allow(dead_code)]

#[macro_export]
macro_rules! impl_entity_data 
{
{
	$entity_type_name:ident <$global_data_name:ty>
	{
		$($datatype:ty:$plural:ident:$mask:ident),+
	}
}=>

{
	pub struct $entity_type_name
	{
		$(
		pub $plural:Vec<$datatype>,
		)+
	}

	impl $crate::Components for $entity_type_name
	{
		fn new()->$entity_type_name
		{
			$entity_type_name
			{
				$(
				$plural:Vec::new(),
				)+
			}
		}
		fn extend(&mut self)
		{
			$(
			self.$plural.push(Default::default());
			)+
		}
	}

	$(
	impl $crate::Component<$datatype> for $crate::World<$entity_type_name,$global_data_name>
	{
		fn has(&self,entity:&$crate::Entity)->bool
		{
			if self.entity_valid(&entity)
			{
				self.components[entity.id]&$mask!=0
			}
			else
			{
				panic!("Attempt to query an invalid entity");
			}
		}

		fn get(&self,entity:&$crate::Entity)->Option<$datatype>
		{
			if self.entity_valid(&entity)
			{
				if self.components[entity.id]&$mask!=0
				{
					Some(self.componentdata.$plural[entity.id].clone())
				}
				else 
				{
					None
				}
			}
			else
			{
				panic!("Attempt to query an invalid entity");
			}
		}

		fn add(&mut self,entity:&$crate::Entity,comp:&$datatype)
		{
			if self.entity_valid(&entity)
			{
				self.componentdata.$plural[entity.id]=comp.clone();
				self.components[entity.id]|=$mask;
			}
			else
			{
				panic!("Attempt to add a component to invalid entity!");
			}
		}

		fn remove(&mut self,entity:&$crate::Entity)
		{
			if self.entity_valid(&entity)
			{
				self.components[entity.id]^=$mask;
			}
			else
			{
				panic!("Attempt to remove a component from invalid entity!");
			}
		}
	}
	)+
}
}

#[derive(Clone,Copy,PartialEq,Eq,Ord,PartialOrd)]
/// The entity id struct. 
pub struct Entity
{
	///Use this to index `World::componentdata` fields
	pub id:usize,
	version:usize
}

///This struct holds everything related to entity-component system.
pub struct World<T,C>
{
	pub entities:Vec<Entity>,
	pub componentdata:T,
	pub globaldata:C,
	recycled_ids:Vec<Entity>,
	entities_to_delete:Vec<Entity>,
	pub components:Vec<u32>,
	next_id:usize,

}

///Trait for systems
pub trait System<T,C>
{
	fn process(&self,entities:Vec<Entity>,world:&mut World<T,C>);
	fn get_interesting_entities(&self,world:&mut World<T,C>)->Vec<Entity>;
	
}

///Internal trait for World::componentdata
pub trait Components
{
	fn new()->Self;
	fn extend(&mut self);
}

///Trait for data not directly associated with entities
pub trait GlobalData
{
	fn new()->Self;
}

impl GlobalData for ()
{
	fn new()->()
	{()}
}

///Trait for component access
pub trait Component<T>
{
	fn has(&self,entity:&Entity)->bool;
	fn get(&self,entity:&Entity)->Option<T>;
	fn add(&mut self,entity:&Entity,comp:&T);
	fn remove(&mut self,entity:&Entity);
}



impl<T:Components,C:GlobalData> World<T,C>
{
	///Creates a new `World`
	pub fn new()->World<T,C>
	{
		World
		{
			componentdata:T::new(),
			globaldata:C::new(),
			entities:Vec::new(),
			recycled_ids:Vec::new(),
			entities_to_delete:Vec::new(),
			components:Vec::new(),
			next_id:0,
		}
	}
	///Adds a new entity
	pub fn add_entity(&mut self)->Entity
	{
		let entity=self.recycled_ids.pop();

		match entity
		{
			Some(e) =>
			{
				self.entities[e.id].version+=1;
				self.entities[e.id];
				self.components[e.id]=1;
				self.entities[e.id]
			}
			None    => 
			{
				let en=Entity{id:self.next_id,version:0};
				self.next_id+=1;
				self.entities.push(en);
				self.components.push(1);
				self.componentdata.extend();
				en
			}
		}
	}
	///Marks an entity for deletion.
	///The entity gets actually deleted the next time you call `update`
	pub fn delete_entity(&mut self,e:&Entity)
	{
		self.entities_to_delete.push(*e);
	}

	///Checks if entity actually exists.
	///Deleted entities also fail this check.
	pub fn entity_valid(&self,e:&Entity)->bool
	{
		self.components[e.id]!=0 && self.entities[e.id].version==e.version
	}

	///Removes entities and runs systems.
	pub fn update(&mut self,systems:&Vec<Box<System<T,C>>>) 
	{
		for e in self.entities_to_delete.iter()
		{
			if self.entity_valid(e)
			{
				self.components[e.id]=0;
				self.recycled_ids.push(self.entities[e.id]);
			}
		}

		for system in systems.iter()
		{
			let entitylist=system.get_interesting_entities(self);
			system.process(entitylist,self);
		}

	}
}


