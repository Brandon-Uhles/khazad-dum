use specs::{prelude::*, saveload::{SimpleMarker, MarkedBuilder, SerializeComponents}, {error::NoError}};
use crate::components::*;
use crate::World;
use std::fs::File;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
            SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
                &( $ecs.read_storage::<$type>(), ),
                &$data.0,
                &$data.1,
                &mut $ser,
            )
            .unwrap();
        )*
    };
}

pub fn save_game(world: &mut World) {
    //helper
    let mapcopy = world.get_mut::<crate::map::Map>().unwrap().clone();
    let savehelper = world
        .create_entity()
        .with(SerializationHelper{map : mapcopy})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    {
        let data = (world.entities(), world.read_storage::<SimpleMarker<SerializeMe>>());

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer =serde_json::Serializer::new(writer);
        serialize_individually!(world, serializer, data, Position, Renderable, Player, Viewshed, Monster,Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, 
            AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
            WantsToDropItem, SerializationHelper);
    }
    world.delete_entity(savehelper).expect("Crash on cleanup");
}