use shipyard::prelude::*;
use futures_signals::signal::{Signal, SignalExt, Mutable};
use lazy_static::lazy_static;
use futures::future::ready;
use futures::executor::block_on;

struct EntityContainer (pub Mutable<Option<EntityId>>);
struct Label (pub String);

lazy_static! {
    pub static ref WORLD:World = {
        let world = World::new::<(Label, )>();
        world.add_unique(EntityContainer(Mutable::new(None)));
        world
    };
}

fn main() {

    //create the signals *before* adding an entity
    //this way the "bad" one will see the old snapshot
    let sig_good = approach_good();
    let sig_bad = approach_bad();

    //for testing, switch this assignment
    let sig = sig_bad;

    //add the entity
    add_entity();

    //spawn the future to run the signal listener
    block_on(sig.for_each(|value| {
        println!("[GOT LABEL] {}", value);
        ready(())
    }));
}

fn add_entity() {
    let entity = WORLD.run::<(EntitiesMut, &mut Label), _, _>(|(mut entities, mut labels)| {
        entities.add_entity(&mut labels, Label("hello!".to_string()))
    });

    WORLD.run::<Unique<&mut EntityContainer>, _, _>(|list| {
        *list.0.lock_mut() = Some(entity);
    });
}

// in the good approach, we get the signal first
// then in its map function we run the world again
fn approach_good() -> impl Signal<Item = String> {
    let container_signal = WORLD.run::<Unique<&EntityContainer>, _, _>(|container| {
        container.0.signal()
    });

    container_signal.map(|entity_id| {
        println!("getting label for entity {:?}", entity_id);
        WORLD.run::<&Label, _, _>(|labels| {
            match entity_id {
                None => "nothing".to_string(),
                Some(entity_id) => (labels).get(entity_id).unwrap().0.to_string()
            }
        })
    })
}

// in the bad approach we run the World
// and then inside of that create our signal (and the map uses the views from same run)
fn approach_bad() -> impl Signal<Item = String> {
    WORLD.run::<(Unique<&EntityContainer>, &Label), _, _>(|(container, labels)| {
        container.0.signal()
            .map(move |entity_id| {
                println!("getting label for entity {:?}", entity_id);
                match entity_id {
                    None => "nothing".to_string(),
                    Some(entity_id) => (&labels).get(entity_id).unwrap().0.to_string()
                }
            })
    })
}
