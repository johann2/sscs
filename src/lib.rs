extern crate rustc_serialize;
use std::mem;

#[macro_export]
macro_rules! impl_entity_data {
{
	$entity_type_name:ident <$global_data_name:ty> 	{
		$($datatype:ty:$plural:ident:$mask:expr),+
	}
}=>

{
	pub struct $entity_type_name {
		$(
		pub $plural:Vec<$datatype>,
		)+
	}

	impl $crate::Components for $entity_type_name {
		fn new()->$entity_type_name {
			$entity_type_name {
				$(
				$plural:Vec::new(),
				)+
			}
		}
		fn extend(&mut self) {
			$(
			self.$plural.push(Default::default());
			)+
		}
	}

	$(
	impl $crate::ComponentAccess<$entity_type_name> for $datatype {
		fn mask()->u32 {
			$mask
		}

		fn get_data(comps:&$entity_type_name)->&Vec<$datatype> {
			&comps.$plural
		}

		fn get_data_mut(comps:&mut $entity_type_name)->&mut Vec<$datatype> {
			&mut comps.$plural
		}
	}
	)+
}
}

#[derive(Clone,Copy,PartialEq,Eq,Ord,PartialOrd,RustcEncodable,RustcDecodable,Default,Hash,Debug)]
/// The entity id struct. 
pub struct Entity {
	id:usize,
	version:usize
}

impl Entity {
	///Use this to index `World::componentdata` fields
	pub fn id(&self)->usize {
		self.id
	}
}

///This struct holds everything related to entity-component system.
pub struct World<T,C> {
	entities:Vec<Entity>,
	pub componentdata:T,
	pub globaldata:C,
	recycled_ids:Vec<Entity>,
	entities_to_delete:Vec<Entity>,
	components:Vec<u32>,
	next_id:usize,
	added:Vec<(u32,Entity)>,
	removed:Vec<(u32,Entity)>
}

///Trait for systems
pub trait System<T,C> {
	///Function that recieves all the entities that have the components specified by `get_entity_mask`.
	fn process(&mut self,entities:Vec<Entity>,world:&mut World<T,C>);
	fn get_entity_mask(&self) -> u32;
	///Function for processing all the entities that got added to this system last frame.
	///Entities added/removed here will get deleted next frame.
	fn process_added(&mut self,entities:Vec<Entity>,world:&mut World<T,C>) {}
	///Function for processing all the entities that got removed from this system last frame.
	///Entities added/removed here will get deleted next frame.
	fn process_removed(&mut self,entities:Vec<Entity>,world:&mut World<T,C>) {}
}

///Internal trait for World::componentdata
pub trait Components {
	fn new() -> Self;
	fn extend(&mut self);
}

///Trait for data not directly associated with entities
pub trait GlobalData {
	fn new() -> Self;
}

impl GlobalData for () {
	fn new() -> ()
	{()}
}

///Trait for components access
pub trait ComponentAccess<T>:Sized {
	///Returns the mask used to check component ownership
	fn mask() -> u32;
	///Returns a reference to the vector holding all components of this type
	fn get_data(comps:&T) -> &Vec<Self>;
	///Returns a mutable reference to the vector holding all components of this type
	fn get_data_mut(comps:&mut T) -> &mut Vec<Self>;
}



impl<T:Components,C:GlobalData> World<T,C> {
	///Creates a new `World`
	pub fn new()->World<T,C> {
		World {
			componentdata:T::new(),
			globaldata:C::new(),
			entities:Vec::new(),
			recycled_ids:Vec::new(),
			entities_to_delete:Vec::new(),
			components:Vec::new(),
			next_id:0,
			added:Vec::new(),
			removed:Vec::new(),
		}
	}
	///Adds a new entity
	pub fn add_entity(&mut self) -> Entity {
		let entity=self.recycled_ids.pop();

		match entity {
			Some(e) => {
				self.entities[e.id].version += 1;
				self.entities[e.id];
				self.components[e.id] = 1;
				self.entities[e.id]
			}
			None    => {
				let en=Entity{id:self.next_id,version:0};
				self.next_id += 1;
				self.entities.push(en);
				self.components.push(1);
				self.componentdata.extend();
				en
			}
		}
	}
	///Marks an entity for deletion.
	///The entity gets actually deleted the next time you call `update`
	pub fn delete_entity(&mut self,e:&Entity) {
		assert!(self.entity_valid(&e));
		self.removed.push((0xFFFFFFFF,*e));
		self.entities_to_delete.push(*e);
	}

	///Checks if entity actually exists.
	///Deleted entities also fail this check.
	pub fn entity_valid(&self,e:&Entity) -> bool {
		self.components[e.id] != 0 && self.entities[e.id].version == e.version
	}

	///Returns all entities that have at least the components specified by `mask`
	pub fn entities_with_components(&self,mask:u32) -> Vec<Entity> {
		let mask2 = mask|1;
		self.entities.iter().filter(|&e| self.components[e.id]&mask2 == mask2).map(|x| *x).collect::<Vec<Entity>>()
	}

	///Returns all valid entities
	pub fn entities(&self) -> Vec<Entity> {
		self.entities_with_components(0)
	}


	///Checks if `entity` has a component of type `Z`
	pub fn has<Z>(&self,entity:&Entity) -> bool 
	where Z:ComponentAccess<T> {
		assert!(self.entity_valid(&entity));
		self.components[entity.id]&Z::mask() != 0
	}

	///Returns a reference to component `Z` attached to `entity
	pub fn get<Z>(&self,entity:&Entity) -> Option<&Z> 
	where Z:ComponentAccess<T> {
		assert!(self.entity_valid(&entity));
		if self.has::<Z>(entity) {
			Some(&Z::get_data(&self.componentdata)[entity.id])
		}
		else {None}
	}

	///Returns a mutable reference to component `Z` attached to `entity`
	pub fn get_mut<Z>(&mut self,entity:&Entity) -> Option<&mut Z> 
	where Z:ComponentAccess<T> {
		assert!(self.entity_valid(&entity));
		if self.has::<Z>(entity) {
			Some(&mut Z::get_data_mut(&mut self.componentdata)[entity.id])
		}
		else {None}
	}

	///Adds a component of type `Z` to `entity`
	///If entity already has component of this type, it gets overwritten.
	pub fn add<Z>(&mut self,entity:&Entity,comp:Z) 
	where Z:ComponentAccess<T> {
		assert!(self.entity_valid(&entity));
		Z::get_data_mut(&mut self.componentdata)[entity.id] = comp;
		if self.components[entity.id] & Z::mask() == 0 {
			self.added.push((Z::mask(),*entity));
		}
		self.components[entity.id] |= Z::mask();
	}

	///Removes a component of type `Z` from `entity`
	///If entity doesn't own component `Z`, this function does nothing.
	pub fn remove<Z>(&mut self,entity:&Entity) 
	where Z:ComponentAccess<T> {
		assert!(self.entity_valid(&entity));
		if self.components[entity.id] & Z::mask() != 0 {
			self.removed.push((Z::mask(),*entity));
			self.components[entity.id] ^= Z::mask();
		}
	}


	///Removes entities marked for deletion and runs systems.
	pub fn update(&mut self,systems:&mut Vec<&mut System<T,C>>) {
		for e in self.entities_to_delete.iter()	{
			if self.entity_valid(e) {
				self.components[e.id] = 0;
				self.recycled_ids.push(self.entities[e.id]);
			}
		}

		//Process
		for system in systems.iter_mut() {
			let mask = system.get_entity_mask();
			let entitylist = self.entities_with_components(mask);
			system.process(entitylist,self);
		}

		let removed_entities=mem::replace(&mut self.removed,Vec::new());
		let added_entities=mem::replace(&mut self.added,Vec::new());

		for system in systems.iter_mut() {
			let sys_mask = system.get_entity_mask();

			let mut added_important_entities:Vec<_>=added_entities.iter()
				.filter(|&&(mask,e)| sys_mask&mask!=0 )
				.map(|&(_,entity)| entity)
				.collect();
			added_important_entities.sort();
			added_important_entities.dedup();

			system.process_added(added_important_entities,self);
		}

		for system in systems.iter_mut() {
			let sys_mask = system.get_entity_mask();

			let mut removed_important_entities:Vec<_>=removed_entities.iter()
				.filter(|&&(mask,e)| sys_mask&mask!=0 )
				.map(|&(_,entity)| entity)
				.collect();

			removed_important_entities.sort();
			removed_important_entities.dedup();
			system.process_removed(removed_important_entities,self);
		}


	}
}


