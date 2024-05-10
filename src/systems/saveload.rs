use crate::components::*;
use crate::World;
use bracket_lib::terminal::Point;
use specs::{
    prelude::*,
    saveload::{
        DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker,
        SimpleMarkerAllocator,
    },
};
use std::fs::File;
use std::path::Path;
use std::{convert::Infallible, fs};

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
            SerializeComponents::<Infallible, SimpleMarker<SerializeMe>>::serialize(
                &( $ecs.read_storage::<$type>(), ),
                &$data.0,
                &$data.1,
                &mut $ser,
            )
            .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $ ( $type:ty),*) => {
        $(
            DeserializeComponents::<Infallible, _>::deserialize(
                &mut (&mut $ecs.write_storage::<$type>(), ),
                &mut $data.0,
                &mut $data.1,
                &mut $data.2,
                &mut $de,
            )
            .unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(world: &mut World) {
    //helper
    let mapcopy = world.get_mut::<crate::map::Map>().unwrap().clone();
    let savehelper = world
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    {
        let data = (
            world.entities(),
            world.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            world,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus
        );
    }
    world.delete_entity(savehelper).expect("Crash on cleanup");
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_world: &mut World) {}

pub fn load_game(world: &mut World) {
    // Delete everything
    {
        let mut to_delete = Vec::new();
        for entity in world.entities().join() {
            to_delete.push(entity);
        }
        for del in to_delete.iter() {
            world.delete_entity(*del).expect("Delete failed");
        }

        let data = fs::read_to_string("./savegame.json").unwrap();
        let mut de = serde_json::Deserializer::from_str(&data);

        {
            let mut d = (
                &mut world.entities(),
                &mut world.write_storage::<SimpleMarker<SerializeMe>>(),
                &mut world.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
            );

            deserialize_individually!(
                world,
                de,
                d,
                Position,
                Renderable,
                Player,
                Viewshed,
                Monster,
                Name,
                BlocksTile,
                CombatStats,
                SufferDamage,
                WantsToMelee,
                Item,
                Consumable,
                Ranged,
                InflictsDamage,
                AreaOfEffect,
                Confusion,
                ProvidesHealing,
                InBackpack,
                WantsToPickupItem,
                WantsToUseItem,
                WantsToDropItem,
                SerializationHelper,
                Equippable,
                Equipped,
                MeleePowerBonus,
                DefenseBonus
            );
        }

        let mut deleteme: Option<Entity> = None;

        {
            let entities = world.entities();
            let helper = world.read_storage::<SerializationHelper>();
            let player = world.read_storage::<Player>();
            let position = world.read_storage::<Position>();

            for (entity, helper) in (&entities, &helper).join() {
                let mut worldmap = world.write_resource::<crate::map::Map>();
                *worldmap = helper.map.clone();
                worldmap.tile_content = vec![Vec::new(); crate::map::MAP_COUNT];
                deleteme = Some(entity);
            }
            for (entity, _player, pos) in (&entities, &player, &position).join() {
                let mut ppos = world.write_resource::<Point>();
                *ppos = Point::new(pos.x, pos.y);
                let mut player_resource = world.write_resource::<Entity>();
                *player_resource = entity;
            }
        }
        world
            .delete_entity(deleteme.unwrap())
            .expect("Unable to delete helper");
    }
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete file")
    }
}
pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}
